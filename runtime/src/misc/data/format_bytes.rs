pub(crate) fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 5] = ["bytes", "kb", "mb", "gb", "tb"];

    let size = bytes as f64;
    let unit = size.log(1024.0).floor() as usize;
    let new_size = size / 1024_f64.powi(unit as i32);

    if unit == 0 {
        format!("{bytes}{}", UNITS[unit])
    } else {
        format!("{new_size:.2}{}", UNITS[unit])
    }
}
