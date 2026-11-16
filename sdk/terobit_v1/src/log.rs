/// Logs a message through the host.
///
/// ```rust
/// use terobit_v1::log;
///
/// log("This message is logged by the host.");
/// ```
pub fn log(message: &str) {
    // SAFETY: the host only reads `len` bytes from `ptr` during the call,
    // and wasm32 pointers and lengths always fit in an i32.
    unsafe {
        terobit_v1_abi::crosspoints::log::log(message.as_ptr() as i32, message.len() as i32);
    }
}
