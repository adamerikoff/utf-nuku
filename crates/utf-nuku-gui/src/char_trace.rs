//! One char's trace, whatever the encoding. The GUI stores a list of these.

use utf_nuku_core::{
    ByteOrder, Utf8CharTrace, Utf16CharTrace, Utf32CharTrace, utf16_units_to_bytes,
    utf32_units_to_bytes,
};

/// Core has 3 different trace structs; this enum lets the GUI keep them in one list.
#[derive(Debug, Clone, PartialEq)]
pub enum CharTrace {
    Utf8(Utf8CharTrace),
    Utf16(Utf16CharTrace),
    Utf32(Utf32CharTrace),
}

impl CharTrace {
    /// The char this trace is about.
    pub fn character(&self) -> char {
        match self {
            CharTrace::Utf8(utf8_trace) => utf8_trace.character,
            CharTrace::Utf16(utf16_trace) => utf16_trace.character,
            CharTrace::Utf32(utf32_trace) => utf32_trace.character,
        }
    }

    /// The char's bytes as they are in a file (byte order only matters for UTF-16/32).
    pub fn character_bytes(&self, byte_order: ByteOrder) -> Vec<u8> {
        match self {
            CharTrace::Utf8(utf8_trace) => utf8_trace.character_bytes.clone(),
            CharTrace::Utf16(utf16_trace) => {
                utf16_units_to_bytes(&utf16_trace.character_units, byte_order)
            }
            CharTrace::Utf32(utf32_trace) => {
                let character_units = [utf32_trace.unit];
                utf32_units_to_bytes(&character_units, byte_order)
            }
        }
    }
}
