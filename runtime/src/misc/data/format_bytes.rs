pub(crate) fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 5] = ["bytes", "kb", "mb", "gb", "tb"];

    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes}{}", UNITS[unit])
    } else {
        format!("{size:.2}{}", UNITS[unit])
    }
}
