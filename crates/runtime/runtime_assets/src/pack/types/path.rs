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

/// checks a path inside a pack against the path rules in the conventions doc
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.len() > MAX_PATH_LEN {
        return Err(format!(
            "path `{path}` must be 1-{MAX_PATH_LEN} bytes, is {}",
            path.len()
        ));
    }

    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() > MAX_PATH_SEGMENTS {
        return Err(format!(
            "path `{path}` has {} segments, at most {MAX_PATH_SEGMENTS} allowed",
            segments.len()
        ));
    }

    for segment in segments {
        if segment.is_empty() || segment.len() > MAX_SEGMENT_LEN {
            return Err(format!(
                "path `{path}` has a segment that isn't 1-{MAX_SEGMENT_LEN} bytes"
            ));
        }
        let allowed = |c: char| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        };
        if !segment.chars().all(allowed) {
            return Err(format!(
                "path `{path}` segment `{segment}` may only use `a-z`, `0-9`, `.`, `_`, `-`"
            ));
        }
        if segment.starts_with('.') || segment.ends_with('.') {
            return Err(format!(
                "path `{path}` segment `{segment}` can't start or end with `.`"
            ));
        }
        let stem = segment.split('.').next().unwrap_or(segment);
        if RESERVED_NAMES.contains(&stem) {
            return Err(format!(
                "path `{path}` segment `{segment}` is a windows reserved name"
            ));
        }
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
