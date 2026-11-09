//! Choices the user makes that aren't core types: encode or decode, hex or bits.

/// What the input box holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// input = text
    Encode,
    /// input = bytes as hex or bits
    Decode,
}

/// How bytes are written in the "all bytes" box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteFormat {
    /// 48 c3 a9
    Hex,
    /// 01001000 11000011 10101001
    Bits,
}
