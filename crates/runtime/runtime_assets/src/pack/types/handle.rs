use std::fmt;
use std::str::FromStr;

/// an asset or script handle, `namespace:id`. always valid, since the only ways to make
/// one check it against the rule in the conventions doc
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle {
    namespace: String,
    id: String,
}

impl Handle {
    pub fn new(namespace: &str, id: &str) -> Result<Self, String> {
        let allowed =
            |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';

        if namespace.is_empty() || !namespace.chars().all(allowed) {
            return Err(format!(
                "handle `{namespace}:{id}` has an invalid namespace"
            ));
        }
        if namespace != "self" {
            return Err(format!(
                "handle `{namespace}:{id}` uses namespace `{namespace}`, only `self` is supported"
            ));
        }
        if id.is_empty() || !id.chars().all(|c| allowed(c) || c == '/') {
            return Err(format!("handle `{namespace}:{id}` has an invalid id"));
        }

        Ok(Self {
            namespace: namespace.to_owned(),
            id: id.to_owned(),
        })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl FromStr for Handle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (namespace, id) = s
            .split_once(':')
            .ok_or_else(|| format!("handle `{s}` is missing a `namespace:` prefix"))?;
        Self::new(namespace, id)
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::Handle;

    #[test]
    fn parses_valid_handles() {
        for s in [
            "self:cube",
            "self:cube/material",
            "self:loot_tables",
            "self:a-1/b_2",
        ] {
            let handle: Handle = s.parse().unwrap();
            assert_eq!(handle.to_string(), s);
        }
    }

    #[test]
    fn splits_namespace_and_id() {
        let handle: Handle = "self:textures/grass".parse().unwrap();
        assert_eq!(handle.namespace(), "self");
        assert_eq!(handle.id(), "textures/grass");
        assert_eq!(handle, Handle::new("self", "textures/grass").unwrap());
    }

    #[test]
    fn rejects_invalid_handles() {
        for s in [
            "cube",
            "self:",
            ":cube",
            "self:cube.material",
            "self:assets:name",
            "self:Cube",
            "other:cube",
            "self:cube material",
        ] {
            assert!(s.parse::<Handle>().is_err(), "{s}");
        }
    }

    #[test]
    fn new_rejects_invalid_parts() {
        assert!(Handle::new("self", "a:b").is_err());
        assert!(Handle::new("", "cube").is_err());
        assert!(Handle::new("self", "").is_err());
    }
}
