//! UTF-8 trace: every step of one char, kept as data (for CLI -v and the GUI).
//!
//! The SAME struct is used by both directions, because encode and decode go through
//! the same values, just in opposite order:
//!
//! ```text
//! encode (utf8_encode::encode_char) ↓        decode (utf8_decode::decode_char) ↑
//! character               é
//! code_point              1110_1001
//! byte_sequence_count     2
//! lead_byte_header        110             (3 bits → 5 data bits left in the lead byte)
//! lead_byte_data          0_0011
//! continuation_bytes_data [10_1001]
//! lead_byte               1100_0011
//! continuation_bytes      [1010_1001]
//! character_bytes         [1100_0011, 1010_1001]
//! ```

/// The steps of one char in UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8CharTrace {
    pub character: char,
    pub code_point: u32,
    /// how many bytes the char has (lead byte included)
    pub byte_sequence_count: usize,
    /// the lead byte header: 0, 110, 1110 or 11110
    pub lead_byte_header: u32,
    /// how many bits the header has: 1, 3, 4 or 5 (the rest of the lead byte is data)
    pub lead_byte_header_bit_count: u32,
    /// the data bits inside the lead byte
    pub lead_byte_data: u32,
    /// 6 data bits per continuation byte
    pub continuation_bytes_data: Vec<u32>,
    /// header + lead_byte_data
    pub lead_byte: u8,
    /// 10 + each continuation byte data
    pub continuation_bytes: Vec<u8>,
    /// all bytes of the char: lead byte, then continuation bytes
    pub character_bytes: Vec<u8>,
}
