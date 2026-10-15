use crate::pack::{
    CHUNK, ChunkEntry, Compression, FileEntry, FileLocation, LARGE_FILE_THRESHOLD, ZSTD_LEVEL,
};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

/// one side's chunked files. the stored chunk bytes live in a temp file, so memory use is
/// bounded by the chunk size however big the pack is
pub(crate) struct SideChunks {
    /// in data section order
    pub entries: Vec<ChunkEntry>,
    /// same length and order as the manifest's `files`
    pub locations: Vec<FileLocation>,
    /// the data section, exactly as it gets written
    pub data: SpoolFile,
}

impl SideChunks {
    /// chunks are in data section order, so the last one ends the section
    pub fn data_len(&self) -> u64 {
        self.entries
            .last()
            .map_or(0, |chunk| chunk.offset + chunk.stored_len as u64)
    }
}

/// a temp file that is deleted when dropped
pub(crate) struct SpoolFile {
    path: PathBuf,
}

impl SpoolFile {
    fn create() -> Result<(Self, File), String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "terobit_pack_{}_{}.data",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let file = File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| format!("`{}`: {e}", path.display()))?;
        info!("Spooling chunk data to {}", path.display());
        Ok((Self { path }, file))
    }

    pub fn open(&self) -> Result<File, String> {
        File::open(&self.path).map_err(|e| format!("`{}`: {e}", self.path.display()))
    }
}

impl Drop for SpoolFile {
    fn drop(&mut self) {
        if std::fs::remove_file(&self.path).is_ok() {
            info!("Removed chunk spool file {}", self.path.display());
        }
    }
}

/// chunks `files` (from `root`) following "writing chunks" in the tbpk v1 doc, so the same
/// content always produces the same chunks
pub(crate) fn chunk_files(
    root: &Path,
    files: &[FileEntry],
    chunk_size: u32,
) -> Result<SideChunks, String> {
    let total: u64 = files.iter().map(|f| f.size).sum();
    info!(
        "Chunking {} files ({total} bytes) from {} with chunk size {chunk_size}",
        files.len(),
        root.display()
    );

    let mut chunker = Chunker::new(chunk_size)?;
    let mut locations = Vec::with_capacity(files.len());

    for entry in files {
        let path = root.join(&entry.path);
        let err = |e: String| format!("`{}`: {e}", entry.path);

        let (location, kind) = if entry.size == 0 {
            let location = FileLocation {
                first_chunk: 0,
                offset_in_chunk: 0,
            };
            (location, "empty")
        } else {
            let file = File::open(&path).map_err(|e| err(e.to_string()))?;
            // a small file bigger than the chunk size can't fit in one chunk, so it's
            // chunked like a large file
            if entry.size >= LARGE_FILE_THRESHOLD || entry.size > chunk_size as u64 {
                (chunker.add_large(file, entry.size).map_err(err)?, "large")
            } else {
                (chunker.add_small(file, entry.size).map_err(err)?, "small")
            }
        };
        info!(
            "Packed {kind} file `{}` ({} bytes), starts in chunk {} at offset {}",
            entry.path, entry.size, location.first_chunk, location.offset_in_chunk
        );
        locations.push(location);
    }

    let (entries, data) = chunker.finish()?;
    let chunks = SideChunks {
        entries,
        locations,
        data,
    };
    info!(
        "Chunked {total} bytes into {} chunks, {} bytes stored",
        chunks.entries.len(),
        chunks.data_len()
    );
    Ok(chunks)
}

struct Chunker {
    chunk_size: u32,
    entries: Vec<ChunkEntry>,
    /// bytes written to the data section so far
    offset: u64,
    /// the open small-file chunk's raw bytes
    open: Vec<u8>,
    spool: SpoolFile,
    out: BufWriter<File>,
}

impl Chunker {
    fn new(chunk_size: u32) -> Result<Self, String> {
        let (spool, file) = SpoolFile::create()?;
        Ok(Self {
            chunk_size,
            entries: Vec::new(),
            offset: 0,
            open: Vec::new(),
            spool,
            out: BufWriter::new(file),
        })
    }

    /// the index the next pushed chunk gets
    fn next_chunk(&self) -> Result<u32, String> {
        u32::try_from(self.entries.len()).map_err(|_| "too many chunks for one pack".into())
    }

    /// starts its own chunks and fills as many as it needs, never sharing the last one
    fn add_large(&mut self, mut file: File, size: u64) -> Result<FileLocation, String> {
        self.close_open()?;
        let location = FileLocation {
            first_chunk: self.next_chunk()?,
            offset_in_chunk: 0,
        };

        let mut buf = Vec::with_capacity(self.chunk_size as usize);
        let mut remaining = size;
        while remaining > 0 {
            let len = remaining.min(self.chunk_size as u64) as usize;
            buf.resize(len, 0);
            file.read_exact(&mut buf).map_err(|e| e.to_string())?;
            self.push_chunk(&buf)?;
            remaining -= len as u64;
        }
        check_eof(&mut file, size)?;

        Ok(location)
    }

    /// goes into the open chunk, closing it first if the file doesn't fit
    fn add_small(&mut self, mut file: File, size: u64) -> Result<FileLocation, String> {
        if self.open.len() as u64 + size > self.chunk_size as u64 {
            self.close_open()?;
        }
        let location = FileLocation {
            first_chunk: self.next_chunk()?,
            offset_in_chunk: self.open.len() as u32,
        };

        let start = self.open.len();
        self.open.resize(start + size as usize, 0);
        file.read_exact(&mut self.open[start..])
            .map_err(|e| e.to_string())?;
        check_eof(&mut file, size)?;

        Ok(location)
    }

    fn close_open(&mut self) -> Result<(), String> {
        if !self.open.is_empty() {
            let raw = std::mem::take(&mut self.open);
            self.push_chunk(&raw)?;
        }
        Ok(())
    }

    /// compresses one chunk and appends it to the data section. stored uncompressed if zstd
    /// saves less than 20%
    fn push_chunk(&mut self, raw: &[u8]) -> Result<(), String> {
        let compressed = zstd::bulk::compress(raw, ZSTD_LEVEL).map_err(|e| e.to_string())?;
        let (stored, compression) = if compressed.len() as u64 * 5 > raw.len() as u64 * 4 {
            (raw, Compression::None)
        } else {
            (&compressed[..], Compression::Zstd)
        };

        self.out.write_all(stored).map_err(|e| e.to_string())?;
        info!(
            "Chunk {}: {} raw bytes, {} stored ({compression:?}) at data offset {}",
            self.entries.len(),
            raw.len(),
            stored.len(),
            self.offset
        );
        self.entries.push(ChunkEntry {
            offset: self.offset,
            stored_len: stored.len() as u32,
            raw_len: raw.len() as u32,
            compression,
            hash: blake3::derive_key(CHUNK, stored),
        });
        self.offset += stored.len() as u64;

        Ok(())
    }

    fn finish(mut self) -> Result<(Vec<ChunkEntry>, SpoolFile), String> {
        self.close_open()?;
        self.out.flush().map_err(|e| e.to_string())?;
        Ok((self.entries, self.spool))
    }
}

/// a file that grew since its size was read would make the manifest's size wrong
fn check_eof(file: &mut File, size: u64) -> Result<(), String> {
    let mut byte = [0u8; 1];
    match file.read(&mut byte).map_err(|e| e.to_string())? {
        0 => Ok(()),
        _ => Err(format!("file changed while packing, expected {size} bytes")),
    }
}

#[cfg(test)]
mod chunker_tests {
    use super::*;

    const CHUNK_SIZE: u32 = 64 * 1024;

    /// writes `files` into a fresh temp dir and returns it with their entries, sorted by path
    fn setup(name: &str, files: &[(&str, Vec<u8>)]) -> (PathBuf, Vec<FileEntry>) {
        let root =
            std::env::temp_dir().join(format!("terobit_chunker_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let mut entries = Vec::new();
        for (path, bytes) in files {
            std::fs::write(root.join(path), bytes).unwrap();
            entries.push(FileEntry {
                path: (*path).to_owned(),
                size: bytes.len() as u64,
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        (root, entries)
    }

    /// bytes zstd can't shrink
    fn noise(len: usize, seed: u64) -> Vec<u8> {
        let mut state = seed | 1;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect()
    }

    fn raw_chunks(chunks: &SideChunks) -> Vec<Vec<u8>> {
        let mut data = Vec::new();
        chunks.data.open().unwrap().read_to_end(&mut data).unwrap();
        assert_eq!(data.len() as u64, chunks.data_len());

        chunks
            .entries
            .iter()
            .map(|chunk| {
                let start = chunk.offset as usize;
                let stored = &data[start..start + chunk.stored_len as usize];
                assert_eq!(blake3::derive_key(CHUNK, stored), chunk.hash);
                match chunk.compression {
                    Compression::None => {
                        assert_eq!(chunk.stored_len, chunk.raw_len);
                        stored.to_vec()
                    }
                    Compression::Zstd => {
                        assert!(chunk.stored_len < chunk.raw_len);
                        zstd::bulk::decompress(stored, chunk.raw_len as usize).unwrap()
                    }
                }
            })
            .collect()
    }

    /// reads every file back the way a reader would, from its location and size
    fn assert_round_trips(root: &Path, files: &[FileEntry], chunks: &SideChunks) {
        let raw = raw_chunks(chunks);
        assert_eq!(chunks.locations.len(), files.len());

        for (entry, location) in files.iter().zip(&chunks.locations) {
            let mut bytes = Vec::new();
            let mut chunk = location.first_chunk as usize;
            let mut offset = location.offset_in_chunk as usize;
            while (bytes.len() as u64) < entry.size {
                let want = entry.size as usize - bytes.len();
                let available = &raw[chunk][offset..];
                bytes.extend_from_slice(&available[..want.min(available.len())]);
                chunk += 1;
                offset = 0;
            }
            assert_eq!(
                bytes,
                std::fs::read(root.join(&entry.path)).unwrap(),
                "{}",
                entry.path
            );
        }
    }

    #[test]
    fn small_files_share_a_chunk() {
        let (root, files) = setup("share", &[("a", vec![1; 100]), ("b", vec![2; 200])]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        assert_eq!(chunks.entries.len(), 1);
        assert_eq!(chunks.locations[0].first_chunk, 0);
        assert_eq!(chunks.locations[0].offset_in_chunk, 0);
        assert_eq!(chunks.locations[1].first_chunk, 0);
        assert_eq!(chunks.locations[1].offset_in_chunk, 100);
        assert!(matches!(chunks.entries[0].compression, Compression::Zstd));
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn small_file_that_doesnt_fit_starts_a_new_chunk() {
        let (root, files) = setup(
            "no_fit",
            &[("a", noise(40 * 1024, 1)), ("b", noise(40 * 1024, 2))],
        );
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        assert_eq!(chunks.entries.len(), 2);
        assert_eq!(chunks.locations[1].first_chunk, 1);
        assert_eq!(chunks.locations[1].offset_in_chunk, 0);
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn large_file_gets_its_own_chunks() {
        let large = noise(LARGE_FILE_THRESHOLD as usize + 10, 3);
        let (root, files) = setup(
            "large",
            &[("a", vec![1; 10]), ("b", large), ("c", vec![3; 10])],
        );
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        // a's chunk, b's 5 chunks (4 full and 10 bytes), then c's chunk
        assert_eq!(chunks.entries.len(), 7);
        assert_eq!(chunks.locations[1].first_chunk, 1);
        assert_eq!(chunks.locations[1].offset_in_chunk, 0);
        assert_eq!(chunks.entries[5].raw_len, 10);
        assert_eq!(chunks.locations[2].first_chunk, 6);
        assert!(matches!(chunks.entries[1].compression, Compression::None));
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn small_file_bigger_than_chunk_size_is_split() {
        let (root, files) = setup("over_chunk", &[("a", noise(CHUNK_SIZE as usize + 1, 4))]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        assert_eq!(chunks.entries.len(), 2);
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn empty_file_reads_no_chunks() {
        let (root, files) = setup("empty", &[("a", vec![1; 10]), ("b", Vec::new())]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        assert_eq!(chunks.entries.len(), 1);
        assert_eq!(chunks.locations[1].first_chunk, 0);
        assert_eq!(chunks.locations[1].offset_in_chunk, 0);
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn no_files_is_empty_data() {
        let (root, files) = setup("none", &[]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        assert!(chunks.entries.is_empty());
        assert_eq!(chunks.data_len(), 0);
    }

    #[test]
    fn same_content_same_chunks() {
        let (root, files) = setup("deterministic", &[("a", noise(300 * 1024, 5))]);
        let first = chunk_files(&root, &files, CHUNK_SIZE).unwrap();
        let second = chunk_files(&root, &files, CHUNK_SIZE).unwrap();

        let hashes = |c: &SideChunks| c.entries.iter().map(|e| e.hash).collect::<Vec<_>>();
        assert_eq!(hashes(&first), hashes(&second));
    }

    #[test]
    fn file_size_mismatch_is_error() {
        let (root, mut files) = setup("mismatch", &[("a", vec![1; 10])]);
        files[0].size = 5;
        assert!(chunk_files(&root, &files, CHUNK_SIZE).is_err());
        files[0].size = 20;
        assert!(chunk_files(&root, &files, CHUNK_SIZE).is_err());
    }

    #[test]
    fn spool_file_is_deleted_on_drop() {
        let (root, files) = setup("drop", &[("a", vec![1; 10])]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE).unwrap();
        let path = chunks.data.path.clone();
        assert!(path.exists());
        drop(chunks);
        assert!(!path.exists());
    }
}
