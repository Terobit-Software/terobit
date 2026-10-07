/// the magic value of the file
pub enum MagicValue {
    /// terobit pack file
    TBPK,
    /// terobit manifest file
    TBMAN,
    /// terobit signature file
    TBSG,
    /// terobit license file
    TBLC,
    /// terobit resource file
    TBRS,
}

pub struct CommonHeader {
    /// the magic value of the file
    pub magic: MagicValue,
    /// the version of the file type, starts at 1
    pub version: u16,
    /// reserved for file type
    pub flags: u16,
    /// length of the header from the start of the file in bytes
    pub header_length: u32,
    /// reserved for file type
    pub reserved: u32,
    /// in bytes the length of the body after the header (file length is `header_length + body_length`)
    pub body_length: u64,
}

/// checks an asset or script id against the `namespace:name` rule in the conventions doc
pub fn validate_id(id: &str) -> Result<(), String> {
    let Some((namespace, name)) = id.split_once(':') else {
        return Err(format!("id `{id}` is missing a `namespace:` prefix"));
    };

    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';

    if namespace.is_empty() || !namespace.chars().all(allowed) {
        return Err(format!("id `{id}` has an invalid namespace"));
    }
    if namespace != "self" {
        return Err(format!(
            "id `{id}` uses namespace `{namespace}`, only `self` is supported"
        ));
    }
    if name.is_empty() || !name.chars().all(|c| allowed(c) || c == '/') {
        return Err(format!("id `{id}` has an invalid name"));
    }

    Ok(())
}

#[cfg(test)]
mod id_tests {
    use super::validate_id;

    #[test]
    fn accepts_valid_ids() {
        for id in [
            "self:cube",
            "self:cube/material",
            "self:loot_tables",
            "self:a-1/b_2",
        ] {
            assert!(validate_id(id).is_ok(), "{id}");
        }
    }

    #[test]
    fn rejects_invalid_ids() {
        for id in [
            "cube",
            "self:",
            ":cube",
            "self:cube.material",
            "self:assets:name",
            "self:Cube",
            "other:cube",
            "self:cube material",
        ] {
            assert!(validate_id(id).is_err(), "{id}");
        }
    }
}
