#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Asks the host to read and apply the command buffer now, instead of
    /// waiting for the end of the cycle. The guest clears the buffer after
    /// this returns.
    ///
    /// `fn(i32, i32)` -> (nothing)
    ///
    /// (pointer to command buffer in guest memory, byte length of buffer) -> (nothing)
    ///
    /// See [`crate::commands`] for the buffer layout.
    pub fn flush_commands(ptr: i32, len: i32);
}
