#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "terobit_v1")]
unsafe extern "C" {
    /// Logs a UTF-8 message to the host.
    ///
    /// `fn(i32, i32)` -> (nothing)
    ///
    /// (pointer to message bytes in guest memory, byte length of message) -> (nothing)
    pub fn log(ptr: i32, len: i32);
}
