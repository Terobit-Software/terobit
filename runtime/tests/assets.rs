// `runtime::assets` and everything in it are `pub(crate)`, so there is
// nothing this external test crate can name or construct; kept as a
// placeholder for parity with the rest of the crate.
#[cfg(test)]
mod tests {
    #[test]
    fn function() {}
}
