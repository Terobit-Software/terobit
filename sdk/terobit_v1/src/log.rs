/// Logs a message through the host.
///
/// ```rust,no_run
/// use terobit_v1::log;
///
/// log("This message is logged by the host.");
/// ```
pub fn log(message: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        crate::outside_sandbox("log");
    }

    // SAFETY: the host only reads `len` bytes from `ptr` during the call,
    // and wasm32 pointers and lengths always fit in an i32.
    #[cfg(target_arch = "wasm32")]
    unsafe {
        terobit_v1_abi::crosspoints::log::log(message.as_ptr() as i32, message.len() as i32);
    }
}
