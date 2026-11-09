//! Layout helper for -v output: labeled rows. (Bit text lives in utf_nuku_core::bits_text.)

/// One line: label in a 16-wide column, then the value.
///
/// ```text
/// row("bytes needed", "2") → "  bytes needed    2"
/// ```
pub fn row(label: &str, value: &str) -> String {
    format!("  {label:<16}{value}")
}
