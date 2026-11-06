#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Logs a UTF-8 message to the host.
    ///
    /// `fn(i32, i32)` -> (nothing)
    ///
    /// (pointer to message bytes in guest memory, byte length of message) -> (nothing)
    ///
    /// ```rust
    /// use terobit_v1::log;
    ///
    /// log("This message is logged by the host.");
    /// ```
    pub fn log(ptr: i32, len: i32);
}
