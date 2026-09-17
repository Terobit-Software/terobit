// `format_bytes` is `pub(crate)` even though `runtime::misc::data` is a
// public module path, so this external test crate has nothing to call;
// kept as a placeholder for parity with the rest of the tree.
#[cfg(test)]
mod tests {
    #[test]
    fn function() {}
}
