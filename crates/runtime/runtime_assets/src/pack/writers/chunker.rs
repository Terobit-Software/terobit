use crate::pack::{CHUNK, ChunkEntry, Compression, FileEntry, FileLocation, LARGE_FILE_THRESHOLD};
use snafu::{OptionExt, ResultExt, Snafu, ensure};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::info;

/// one side's files couldn't be chunked
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ChunkError {
    #[snafu(display(
        "failed to create the temp spool file `{}` that holds chunk data while packing",
        path.display()
    ))]
    CreateSpool {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "failed to open source file `{pack_path}` at `{}` for chunking",
        full_path.display()
    ))]
    OpenSource {
        pack_path: String,
        full_path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "failed to chunk {kind} file `{pack_path}` ({size} bytes), file {number} of {total}"
    ))]
    AddFile {
        pack_path: String,
        kind: &'static str,
        size: u64,
        number: usize,
        total: usize,
        #[snafu(source(from(ChunkerError, Box::new)))]
        source: Box<ChunkerError>,
    },

    #[snafu(display("failed to finish the last chunk and flush the spool file"))]
    Finish {
        #[snafu(source(from(ChunkerError, Box::new)))]
        source: Box<ChunkerError>,
    },
}

/// the chunker failed while adding bytes or writing a chunk
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ChunkerError {
    #[snafu(display(
        "failed to read {len} bytes at offset {offset} of the source file, it may have shrunk since its size was read"
    ))]
    ReadSource {
        offset: u64,
        len: usize,
        source: std::io::Error,
    },

    #[snafu(display("failed to check the source file ended after {size} bytes"))]
    CheckEof { size: u64, source: std::io::Error },

    #[snafu(display(
        "source file is longer than the {size} bytes its size was read as, it changed while packing"
    ))]
    FileGrew { size: u64 },

    #[snafu(display("chunk {index} would be past the most a pack can index ({})", u32::MAX))]
    TooManyChunks { index: usize },

    #[snafu(display("failed to zstd compress chunk {index} ({len} raw bytes) at level {level}"))]
    Compress {
        index: usize,
        len: usize,
        level: i32,
        source: std::io::Error,
    },

    #[snafu(display(
        "failed to write chunk {index} ({len} stored bytes) to spool file `{}` at data offset {offset}",
        path.display()
    ))]
    WriteChunk {
        index: usize,
        len: usize,
        offset: u64,
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to flush spool file `{}`", path.display()))]
    FlushSpool {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// MiB/s for `bytes` over `elapsed`, for the timing logs
fn throughput(bytes: u64, elapsed: Duration) -> f64 {
    let secs = elapsed.as_secs_f64();
    if secs == 0.0 {
        return f64::INFINITY;
    }
    bytes as f64 / (1024.0 * 1024.0) / secs
}

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
    fn create() -> Result<(Self, File), ChunkError> {
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
            .context(CreateSpoolSnafu { path: &path })?;
        info!("Spooling chunk data to {}", path.display());
        Ok((Self { path }, file))
    }

    pub fn open(&self) -> std::io::Result<File> {
        File::open(&self.path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SpoolFile {
    fn drop(&mut self) {
        match std::fs::remove_file(&self.path) {
            Ok(()) => info!("Removed chunk spool file {}", self.path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => tracing::warn!(
                "Failed to remove chunk spool file {}, it is left in the temp directory and can be deleted by hand: {e}",
                self.path.display()
            ),
        }
    }
}

/// chunks `files` (from `root`) following "writing chunks" in the tbpk v1 doc, so the same
/// content always produces the same chunks
pub(crate) fn chunk_files(
    root: &Path,
    files: &[FileEntry],
    chunk_size: u32,
    zstd_level: Option<i32>,
) -> Result<SideChunks, ChunkError> {
    let started = Instant::now();
    let total: u64 = files.iter().map(|f| f.size).sum();
    info!(
        "Chunking {} files ({total} bytes) from {} with chunk size {chunk_size}, zstd level {zstd_level:?}",
        files.len(),
        root.display()
    );

    let setup_started = Instant::now();
    let mut chunker = Chunker::new(chunk_size, zstd_level)?;
    let mut locations = Vec::with_capacity(files.len());
    info!("Chunker setup took {:?}", setup_started.elapsed());

    let mut open_time = Duration::ZERO;
    let mut add_time = Duration::ZERO;
    for (index, entry) in files.iter().enumerate() {
        let file_started = Instant::now();
        let path = root.join(&entry.path);
        info!(
            "File {}/{}: starting `{}` ({} bytes), chunks so far {}, open chunk has {} bytes",
            index + 1,
            files.len(),
            entry.path,
            entry.size,
            chunker.entries.len(),
            chunker.open.len()
        );

        let (location, kind) = if entry.size == 0 {
            let location = FileLocation {
                first_chunk: 0,
                offset_in_chunk: 0,
            };
            (location, "empty")
        } else {
            let open_started = Instant::now();
            let file = File::open(&path).context(OpenSourceSnafu {
                pack_path: &entry.path,
                full_path: &path,
            })?;
            let opened = open_started.elapsed();
            open_time += opened;
            info!("File `{}`: open took {opened:?}", entry.path);

            let add_started = Instant::now();
            // a small file bigger than the chunk size can't fit in one chunk, so it's
            // chunked like a large file
            let large = entry.size >= LARGE_FILE_THRESHOLD || entry.size > chunk_size as u64;
            let kind = if large { "large" } else { "small" };
            let added = if large {
                chunker.add_large(file, entry.size)
            } else {
                chunker.add_small(file, entry.size)
            };
            let result = (
                added.context(AddFileSnafu {
                    pack_path: &entry.path,
                    kind,
                    size: entry.size,
                    number: index + 1,
                    total: files.len(),
                })?,
                kind,
            );
            let added = add_started.elapsed();
            add_time += added;
            info!(
                "File `{}`: add as {} took {added:?} ({:.2} MiB/s)",
                entry.path,
                result.1,
                throughput(entry.size, added)
            );
            result
        };
        let file_elapsed = file_started.elapsed();
        info!(
            "Packed {kind} file `{}` ({} bytes), starts in chunk {} at offset {}, took {file_elapsed:?} ({:.2} MiB/s)",
            entry.path,
            entry.size,
            location.first_chunk,
            location.offset_in_chunk,
            throughput(entry.size, file_elapsed)
        );
        locations.push(location);
    }

    let finish_started = Instant::now();
    let (entries, data, stats) = chunker.finish().context(FinishSnafu)?;
    info!("Chunker finish took {:?}", finish_started.elapsed());
    let chunks = SideChunks {
        entries,
        locations,
        data,
    };
    let elapsed = started.elapsed();
    info!(
        "Chunked {total} bytes into {} chunks, {} bytes stored, took {elapsed:?} ({:.2} MiB/s)",
        chunks.entries.len(),
        chunks.data_len(),
        throughput(total, elapsed)
    );
    info!(
        "Chunking time breakdown: file open {open_time:?}, file add {add_time:?} \
         (read {:?}, compress {:?}, hash {:?}, write {:?}, flush {:?})",
        stats.read, stats.compress, stats.hash, stats.write, stats.flush
    );
    if let Some(avg) = stats.chunk_total.checked_div(chunks.entries.len() as u32) {
        info!(
            "Per chunk push: total {:?}, average {avg:?}, slowest {:?} (chunk {}), fastest {:?} (chunk {})",
            stats.chunk_total, stats.slowest.1, stats.slowest.0, stats.fastest.1, stats.fastest.0
        );
    }
    Ok(chunks)
}

struct Chunker {
    chunk_size: u32,
    /// `None` stores every chunk uncompressed
    zstd_level: Option<i32>,
    entries: Vec<ChunkEntry>,
    /// bytes written to the data section so far
    offset: u64,
    /// the open small-file chunk's raw bytes
    open: Vec<u8>,
    spool: SpoolFile,
    out: BufWriter<File>,
    /// when the open small-file chunk got its first file, how long reading into it has taken,
    /// and how many files it holds
    open_since: Option<Instant>,
    open_read: Duration,
    open_files: u32,
    stats: ChunkStats,
}

/// accumulated timings, logged once chunking finishes
#[derive(Default)]
struct ChunkStats {
    read: Duration,
    compress: Duration,
    hash: Duration,
    write: Duration,
    flush: Duration,
    /// sum of every push_chunk call
    chunk_total: Duration,
    /// (chunk index, push time)
    slowest: (usize, Duration),
    fastest: (usize, Duration),
}

impl Chunker {
    fn new(chunk_size: u32, zstd_level: Option<i32>) -> Result<Self, ChunkError> {
        let (spool, file) = SpoolFile::create()?;
        Ok(Self {
            chunk_size,
            zstd_level,
            entries: Vec::new(),
            offset: 0,
            open: Vec::new(),
            spool,
            out: BufWriter::new(file),
            open_since: None,
            open_read: Duration::ZERO,
            open_files: 0,
            stats: ChunkStats {
                fastest: (0, Duration::MAX),
                ..Default::default()
            },
        })
    }

    /// the index the next pushed chunk gets
    fn next_chunk(&self) -> Result<u32, ChunkerError> {
        let index = self.entries.len();
        u32::try_from(index)
            .ok()
            .context(TooManyChunksSnafu { index })
    }

    /// starts its own chunks and fills as many as it needs, never sharing the last one
    fn add_large(&mut self, mut file: File, size: u64) -> Result<FileLocation, ChunkerError> {
        self.close_open()?;
        let location = FileLocation {
            first_chunk: self.next_chunk()?,
            offset_in_chunk: 0,
        };

        let mut buf = Vec::with_capacity(self.chunk_size as usize);
        let mut remaining = size;
        let mut part = 0u64;
        let parts = size.div_ceil(self.chunk_size as u64);
        while remaining > 0 {
            let chunk_started = Instant::now();
            let len = remaining.min(self.chunk_size as u64) as usize;
            buf.resize(len, 0);
            let read_started = Instant::now();
            file.read_exact(&mut buf).context(ReadSourceSnafu {
                offset: size - remaining,
                len,
            })?;
            let read = read_started.elapsed();
            self.stats.read += read;
            info!(
                "Large file part {}/{parts}: read {len} bytes in {read:?} ({:.2} MiB/s)",
                part + 1,
                throughput(len as u64, read)
            );
            self.push_chunk(&buf)?;
            info!(
                "Large file part {}/{parts}: read + push took {:?}",
                part + 1,
                chunk_started.elapsed()
            );
            remaining -= len as u64;
            part += 1;
        }
        let eof_started = Instant::now();
        check_eof(&mut file, size)?;
        info!("Large file eof check took {:?}", eof_started.elapsed());

        Ok(location)
    }

    /// goes into the open chunk, closing it first if the file doesn't fit
    fn add_small(&mut self, mut file: File, size: u64) -> Result<FileLocation, ChunkerError> {
        if self.open.len() as u64 + size > self.chunk_size as u64 {
            info!(
                "Small file ({size} bytes) doesn't fit in open chunk ({} bytes used of {}), closing it",
                self.open.len(),
                self.chunk_size
            );
            self.close_open()?;
        }
        if self.open_since.is_none() {
            info!("Opening small-file chunk {}", self.entries.len());
            self.open_since = Some(Instant::now());
        }
        let location = FileLocation {
            first_chunk: self.next_chunk()?,
            offset_in_chunk: self.open.len() as u32,
        };

        let start = self.open.len();
        let resize_started = Instant::now();
        self.open.resize(start + size as usize, 0);
        let resized = resize_started.elapsed();
        let read_started = Instant::now();
        file.read_exact(&mut self.open[start..])
            .context(ReadSourceSnafu {
                offset: 0u64,
                len: size as usize,
            })?;
        let read = read_started.elapsed();
        let eof_started = Instant::now();
        check_eof(&mut file, size)?;
        let eof = eof_started.elapsed();
        self.stats.read += read;
        self.open_read += read;
        self.open_files += 1;
        info!(
            "Small file into chunk {} at offset {start}: resize {resized:?}, read {size} bytes in {read:?} ({:.2} MiB/s), eof check {eof:?}, chunk now {} bytes",
            location.first_chunk,
            throughput(size, read),
            self.open.len()
        );

        Ok(location)
    }

    fn close_open(&mut self) -> Result<(), ChunkerError> {
        if !self.open.is_empty() {
            let raw = std::mem::take(&mut self.open);
            let files = std::mem::take(&mut self.open_files);
            let read = std::mem::take(&mut self.open_read);
            let index = self.entries.len();
            info!(
                "Closing small-file chunk {index}: {files} files, {} raw bytes, read in {read:?}",
                raw.len()
            );
            self.push_chunk(&raw)?;
            if let Some(since) = self.open_since.take() {
                info!(
                    "Small-file chunk {index}: open to closed took {:?}",
                    since.elapsed()
                );
            }
        }
        Ok(())
    }

    /// compresses one chunk and appends it to the data section. stored uncompressed if zstd
    /// saves less than 20%, or if there's no zstd level
    fn push_chunk(&mut self, raw: &[u8]) -> Result<(), ChunkerError> {
        let started = Instant::now();
        let index = self.entries.len();
        info!("Chunk {index}: pushing {} raw bytes", raw.len());

        let compress_started = Instant::now();
        let compressed = match self.zstd_level {
            Some(level) => {
                let compressed = zstd::bulk::compress(raw, level).context(CompressSnafu {
                    index,
                    len: raw.len(),
                    level,
                })?;
                let elapsed = compress_started.elapsed();
                info!(
                    "Chunk {index}: zstd level {level} compress took {elapsed:?} ({:.2} MiB/s), {} -> {} bytes ({:.1}%)",
                    throughput(raw.len() as u64, elapsed),
                    raw.len(),
                    compressed.len(),
                    compressed.len() as f64 * 100.0 / raw.len().max(1) as f64
                );
                Some(compressed)
            }
            None => {
                info!("Chunk {index}: compression is off, storing uncompressed");
                None
            }
        };
        let compress = compress_started.elapsed();
        let (stored, compression) = match &compressed {
            Some(compressed) if compressed.len() as u64 * 5 <= raw.len() as u64 * 4 => {
                (&compressed[..], Compression::Zstd)
            }
            Some(_) => {
                info!("Chunk {index}: zstd saved less than 20%, storing uncompressed");
                (raw, Compression::None)
            }
            None => (raw, Compression::None),
        };

        let write_started = Instant::now();
        self.out.write_all(stored).context(WriteChunkSnafu {
            index,
            len: stored.len(),
            offset: self.offset,
            path: self.spool.path(),
        })?;
        let write = write_started.elapsed();
        info!(
            "Chunk {index}: write of {} bytes took {write:?} ({:.2} MiB/s)",
            stored.len(),
            throughput(stored.len() as u64, write)
        );

        let hash_started = Instant::now();
        let hash = blake3::derive_key(CHUNK, stored);
        let hash_time = hash_started.elapsed();
        info!(
            "Chunk {index}: blake3 hash took {hash_time:?} ({:.2} MiB/s)",
            throughput(stored.len() as u64, hash_time)
        );

        info!(
            "Chunk {index}: {} raw bytes, {} stored ({compression:?}) at data offset {}",
            raw.len(),
            stored.len(),
            self.offset
        );
        self.entries.push(ChunkEntry {
            offset: self.offset,
            stored_len: stored.len() as u32,
            raw_len: raw.len() as u32,
            compression,
            hash,
        });
        self.offset += stored.len() as u64;

        let total = started.elapsed();
        info!(
            "Chunk {index}: push took {total:?} ({:.2} MiB/s raw) [compress {compress:?}, write {write:?}, hash {hash_time:?}]",
            throughput(raw.len() as u64, total)
        );
        self.stats.compress += compress;
        self.stats.write += write;
        self.stats.hash += hash_time;
        self.stats.chunk_total += total;
        if total > self.stats.slowest.1 {
            self.stats.slowest = (index, total);
        }
        if total < self.stats.fastest.1 {
            self.stats.fastest = (index, total);
        }

        Ok(())
    }

    fn finish(mut self) -> Result<(Vec<ChunkEntry>, SpoolFile, ChunkStats), ChunkerError> {
        self.close_open()?;
        let flush_started = Instant::now();
        self.out.flush().context(FlushSpoolSnafu {
            path: self.spool.path(),
        })?;
        self.stats.flush = flush_started.elapsed();
        info!("Spool flush took {:?}", self.stats.flush);
        Ok((self.entries, self.spool, self.stats))
    }
}

/// a file that grew since its size was read would make the manifest's size wrong
fn check_eof(file: &mut File, size: u64) -> Result<(), ChunkerError> {
    let mut byte = [0u8; 1];
    let read = file.read(&mut byte).context(CheckEofSnafu { size })?;
    ensure!(read == 0, FileGrewSnafu { size });
    Ok(())
}

#[cfg(test)]
mod chunker_tests {
    use super::*;
    use crate::pack::ZSTD_LEVEL;

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
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

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
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

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
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

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
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

        assert_eq!(chunks.entries.len(), 2);
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn empty_file_reads_no_chunks() {
        let (root, files) = setup("empty", &[("a", vec![1; 10]), ("b", Vec::new())]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

        assert_eq!(chunks.entries.len(), 1);
        assert_eq!(chunks.locations[1].first_chunk, 0);
        assert_eq!(chunks.locations[1].offset_in_chunk, 0);
        assert_round_trips(&root, &files, &chunks);
    }

    #[test]
    fn no_files_is_empty_data() {
        let (root, files) = setup("none", &[]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

        assert!(chunks.entries.is_empty());
        assert_eq!(chunks.data_len(), 0);
    }

    #[test]
    fn same_content_same_chunks() {
        let (root, files) = setup("deterministic", &[("a", noise(300 * 1024, 5))]);
        let first = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();
        let second = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();

        let hashes = |c: &SideChunks| c.entries.iter().map(|e| e.hash).collect::<Vec<_>>();
        assert_eq!(hashes(&first), hashes(&second));
    }

    #[test]
    fn file_size_mismatch_is_error() {
        let (root, mut files) = setup("mismatch", &[("a", vec![1; 10])]);
        files[0].size = 5;
        let err = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL))
            .err()
            .expect("a file longer than its size should be rejected");
        assert!(
            matches!(&err, ChunkError::AddFile { source, .. }
                if matches!(**source, ChunkerError::FileGrew { size: 5 })),
            "{err:?}"
        );
        files[0].size = 20;
        let err = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL))
            .err()
            .expect("a file shorter than its size should be rejected");
        assert!(
            matches!(&err, ChunkError::AddFile { source, .. }
                if matches!(**source, ChunkerError::ReadSource { .. })),
            "{err:?}"
        );
    }

    #[test]
    fn spool_file_is_deleted_on_drop() {
        let (root, files) = setup("drop", &[("a", vec![1; 10])]);
        let chunks = chunk_files(&root, &files, CHUNK_SIZE, Some(ZSTD_LEVEL)).unwrap();
        let path = chunks.data.path.clone();
        assert!(path.exists());
        drop(chunks);
        assert!(!path.exists());
    }
}
