use snafu::{Snafu, ensure};

/// most bytes a whole path can take
pub const MAX_PATH_LEN: usize = 255;
/// most segments a path can have
pub const MAX_PATH_SEGMENTS: usize = 16;
/// most bytes one segment can take
pub const MAX_SEGMENT_LEN: usize = 64;

/// names windows reserves, with or without an extension
const RESERVED_NAMES: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// why a path inside a pack broke the path rules in the conventions doc
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum PathError {
    #[snafu(display(
        "pack path `{path}` is {len} bytes long, but must be 1 to {MAX_PATH_LEN} bytes"
    ))]
    Length { path: String, len: usize },

    #[snafu(display(
        "pack path `{path}` has {segments} `/` separated segments, but at most {MAX_PATH_SEGMENTS} are allowed"
    ))]
    TooManySegments { path: String, segments: usize },

    #[snafu(display(
        "pack path `{path}` has segment #{index} (`{segment}`) that is {len} bytes long, but each segment must be 1 to {MAX_SEGMENT_LEN} bytes (empty segments come from a leading, trailing or doubled `/`)"
    ))]
    SegmentLength {
        path: String,
        index: usize,
        segment: String,
        len: usize,
    },

    #[snafu(display(
        "pack path `{path}` segment `{segment}` contains {bad:?}, but only lowercase `a-z`, `0-9`, `.`, `_` and `-` are allowed"
    ))]
    InvalidChar {
        path: String,
        segment: String,
        bad: char,
    },

    #[snafu(display(
        "pack path `{path}` segment `{segment}` starts or ends with `.`, which would make it hidden, relative (`.`/`..`) or ambiguous on windows"
    ))]
    DotEdge { path: String, segment: String },

    #[snafu(display(
        "pack path `{path}` segment `{segment}` uses the windows reserved name `{stem}`, which can't be extracted on windows"
    ))]
    ReservedName {
        path: String,
        segment: String,
        stem: String,
    },
}

/// checks a path inside a pack against the path rules in the conventions doc
pub fn validate_path(path: &str) -> Result<(), PathError> {
    ensure!(
        !path.is_empty() && path.len() <= MAX_PATH_LEN,
        LengthSnafu {
            path,
            len: path.len()
        }
    );

    let segments = path.split('/').collect::<Vec<_>>();
    ensure!(
        segments.len() <= MAX_PATH_SEGMENTS,
        TooManySegmentsSnafu {
            path,
            segments: segments.len()
        }
    );

    for (index, segment) in segments.into_iter().enumerate() {
        ensure!(
            !segment.is_empty() && segment.len() <= MAX_SEGMENT_LEN,
            SegmentLengthSnafu {
                path,
                index,
                segment,
                len: segment.len()
            }
        );
        let allowed = |c: char| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        };
        if let Some(bad) = segment.chars().find(|&c| !allowed(c)) {
            return InvalidCharSnafu { path, segment, bad }.fail();
        }
        ensure!(
            !segment.starts_with('.') && !segment.ends_with('.'),
            DotEdgeSnafu { path, segment }
        );
        let stem = segment.split('.').next().unwrap_or(segment);
        ensure!(
            !RESERVED_NAMES.contains(&stem),
            ReservedNameSnafu {
                path,
                segment,
                stem
            }
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_paths() {
        for path in [
            "a",
            "meshes/mesh.obj",
            "textures/grass.png",
            "a-b_c/d.e.f",
            "console.png",
            "com10.txt",
        ] {
            assert!(validate_path(path).is_ok(), "{path}");
        }
    }

    #[test]
    fn rejects_invalid_paths() {
        for path in [
            "",
            "/a",
            "a/",
            "a//b",
            "A.png",
            "a b",
            "a\\b",
            "é",
            ".",
            "..",
            "../a",
            ".hidden",
            "trailing.",
            "con",
            "con.png",
            "dir/lpt1.txt",
            "nul.tar.gz",
        ] {
            assert!(validate_path(path).is_err(), "{path}");
        }
    }

    #[test]
    fn enforces_length_limits() {
        assert!(validate_path(&"a".repeat(MAX_SEGMENT_LEN)).is_ok());
        assert!(validate_path(&"a".repeat(MAX_SEGMENT_LEN + 1)).is_err());

        assert!(validate_path(&["a"; MAX_PATH_SEGMENTS].join("/")).is_ok());
        assert!(validate_path(&["a"; MAX_PATH_SEGMENTS + 1].join("/")).is_err());

        // 4 full segments plus separators is 259 bytes
        assert!(validate_path(&vec!["a".repeat(MAX_SEGMENT_LEN); 4].join("/")).is_err());
    }
}
