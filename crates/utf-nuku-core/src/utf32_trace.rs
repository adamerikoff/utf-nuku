//! UTF-32 trace: every step of one char, kept as data (for CLI -v and the GUI).
//! UTF-32 has only 1 step: the unit IS the code point. Same struct for encode and decode.

/// The steps of one char in UTF-32.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf32CharTrace {
    pub character: char,
    pub code_point: u32,
    /// the char's only unit: the code point itself
    pub unit: u32,
}
