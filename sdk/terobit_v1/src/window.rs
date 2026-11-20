/// Sets the primary window's title through the host.
///
/// ```rust
/// use terobit_v1::set_title;
///
/// set_title("My Game");
/// ```
pub fn set_title(title: &str) {
    // SAFETY: the host only reads `len` bytes from `ptr` during the call,
    // and wasm32 pointers and lengths always fit in an i32.
    unsafe {
        terobit_v1_abi::crosspoints::window::set_title(title.as_ptr() as i32, title.len() as i32);
    }
}
