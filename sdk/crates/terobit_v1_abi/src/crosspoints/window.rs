#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "terobit_v1")]
unsafe extern "C" {
    /// Sets the primary window's title to a UTF-8 string.
    ///
    /// `fn(i32, i32)` -> (nothing)
    ///
    /// (pointer to title bytes in guest memory, byte length of title) -> (nothing)
    pub fn set_title(ptr: i32, len: i32);
}
