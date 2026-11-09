//! UTF-16 trace: every step of one char, kept as data (for CLI -v and the GUI).
//!
//! The SAME struct is used by both directions (utf16_encode ↓, utf16_decode ↑):
//!
//! ```text
//! single unit (€):                          surrogate pair (😀):
//! code_point           0010_0000_1010_1100  code_point           0001_1111_0110_0000_0000
//! unit_sequence_count  1                    unit_sequence_count  2
//! surrogate_pair       None                 surrogate_pair_data  0000_1111_0110_0000_0000 (− 0x10000)
//! character_units      [0010_0000_1010_1100] high_surrogate_data 00_0011_1101             (left 10)
//!                                           low_surrogate_data   10_0000_0000             (right 10)
//!                                           high_surrogate       1101_1000_0011_1101      (110110 + high)
//!                                           low_surrogate        1101_1110_0000_0000      (110111 + low)
//! ```

/// The pair steps. Only exists for chars above 0xFFFF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurrogatePairTrace {
    /// code point − 0x10000 (exactly 20 bits)
    pub surrogate_pair_data: u32,
    /// left 10 bits
    pub high_surrogate_data: u32,
    /// right 10 bits
    pub low_surrogate_data: u32,
    /// 110110 + high_surrogate_data
    pub high_surrogate: u16,
    /// 110111 + low_surrogate_data
    pub low_surrogate: u16,
}

/// The steps of one char in UTF-16.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf16CharTrace {
    pub character: char,
    pub code_point: u32,
    /// 1 (single unit) or 2 (surrogate pair)
    pub unit_sequence_count: usize,
    /// None for a single unit, Some(steps) for a pair
    pub surrogate_pair: Option<SurrogatePairTrace>,
    /// all units of the char: 1 or 2
    pub character_units: Vec<u16>,
}
