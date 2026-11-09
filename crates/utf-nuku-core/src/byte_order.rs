//! Byte order: turn 16/32-bit units into bytes and back.
//!
//! Files and pipes only carry bytes (u8). A u16 unit is 2 bytes, a u32 unit is 4 bytes.
//! Byte order = which byte goes into the file first. The bits inside a byte never change.
//!
//! ```text
//! unit 1101_1000_0011_1101
//!   Big    → [1101_1000, 0011_1101]   left byte first (readable, header byte first)
//!   Little → [0011_1101, 1101_1000]   right byte first (how modern CPUs store it)
//! ```
//!
//! UTF-8 has no byte order: its unit is already 1 byte.
//!
//! Rust has the conversions built in: `to_be_bytes`, `to_le_bytes`,
//! `from_be_bytes`, `from_le_bytes`. This file only picks the right one.

use crate::errors::DecodeError;

/// Which byte of a unit goes first.
/// `Clone, Copy` → it can be copied like a number, so passing it around never moves it.
/// `Debug, PartialEq, Eq` → needed for `assert_eq!` in tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// right byte first
    Little,
    /// left byte first
    Big,
}

// ---------- UTF-16: u16 ↔ 2 bytes ----------

/// Split every u16 unit into 2 bytes.
///
/// ```text
/// [1101_1000_0011_1101], Big → [1101_1000, 0011_1101]
/// ```
pub fn utf16_units_to_bytes(utf16_units: &[u16], byte_order: ByteOrder) -> Vec<u8> {
    let mut utf16_bytes = Vec::new();

    for &unit in utf16_units {
        // [u8; 2] = an array of exactly 2 bytes
        let unit_bytes = match byte_order {
            ByteOrder::Little => unit.to_le_bytes(),
            ByteOrder::Big => unit.to_be_bytes(),
        };
        utf16_bytes.extend(unit_bytes);
    }

    utf16_bytes
}

/// Glue every 2 bytes into a u16 unit.
/// Error if the byte count isn't a multiple of 2 (the last unit is cut off).
///
/// ```text
/// [1101_1000, 0011_1101], Big → [1101_1000_0011_1101]
/// ```
pub fn bytes_to_utf16_units(
    utf16_bytes: &[u8],
    byte_order: ByteOrder,
) -> Result<Vec<u16>, DecodeError> {
    // % = remainder: 3 % 2 = 1 → one byte left over
    let leftover_byte_count = utf16_bytes.len() % 2;
    if leftover_byte_count != 0 {
        return Err(DecodeError::UnexpectedEnd);
    }

    let mut utf16_units = Vec::new();

    // chunks_exact(2) walks the slice 2 bytes at a time: [b0, b1], [b2, b3], …
    for unit_bytes in utf16_bytes.chunks_exact(2) {
        // from_*_bytes wants an array, so copy the 2-byte slice into one
        let unit_byte_array = [unit_bytes[0], unit_bytes[1]];
        let unit = match byte_order {
            ByteOrder::Little => u16::from_le_bytes(unit_byte_array),
            ByteOrder::Big => u16::from_be_bytes(unit_byte_array),
        };
        utf16_units.push(unit);
    }

    Ok(utf16_units)
}

// ---------- UTF-32: u32 ↔ 4 bytes ----------

/// Split every u32 unit into 4 bytes.
///
/// ```text
/// [0000_0000_0000_0001_1111_0110_0000_0000], Big → [0000_0000, 0000_0001, 1111_0110, 0000_0000]
/// ```
pub fn utf32_units_to_bytes(utf32_units: &[u32], byte_order: ByteOrder) -> Vec<u8> {
    let mut utf32_bytes = Vec::new();

    for &unit in utf32_units {
        // [u8; 4] = an array of exactly 4 bytes
        let unit_bytes = match byte_order {
            ByteOrder::Little => unit.to_le_bytes(),
            ByteOrder::Big => unit.to_be_bytes(),
        };
        utf32_bytes.extend(unit_bytes);
    }

    utf32_bytes
}

/// Glue every 4 bytes into a u32 unit.
/// Error if the byte count isn't a multiple of 4 (the last unit is cut off).
pub fn bytes_to_utf32_units(
    utf32_bytes: &[u8],
    byte_order: ByteOrder,
) -> Result<Vec<u32>, DecodeError> {
    let leftover_byte_count = utf32_bytes.len() % 4;
    if leftover_byte_count != 0 {
        return Err(DecodeError::UnexpectedEnd);
    }

    let mut utf32_units = Vec::new();

    // 4 bytes at a time: [b0, b1, b2, b3], [b4, b5, b6, b7], …
    for unit_bytes in utf32_bytes.chunks_exact(4) {
        let unit_byte_array = [unit_bytes[0], unit_bytes[1], unit_bytes[2], unit_bytes[3]];
        let unit = match byte_order {
            ByteOrder::Little => u32::from_le_bytes(unit_byte_array),
            ByteOrder::Big => u32::from_be_bytes(unit_byte_array),
        };
        utf32_units.push(unit);
    }

    Ok(utf32_units)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 😀 in UTF-16: high surrogate, low surrogate
    const SMILE_UTF16: [u16; 2] = [0b1101_1000_0011_1101, 0b1101_1110_0000_0000];
    // 😀 in UTF-32: the code point
    const SMILE_UTF32: [u32; 1] = [0b0000_0000_0000_0001_1111_0110_0000_0000];

    // ---------- UTF-16 ----------

    #[test]
    fn utf16_big_puts_left_byte_first() {
        assert_eq!(
            utf16_units_to_bytes(&SMILE_UTF16, ByteOrder::Big),
            [0b1101_1000, 0b0011_1101, 0b1101_1110, 0b0000_0000]
        );
    }

    #[test]
    fn utf16_little_puts_right_byte_first() {
        assert_eq!(
            utf16_units_to_bytes(&SMILE_UTF16, ByteOrder::Little),
            [0b0011_1101, 0b1101_1000, 0b0000_0000, 0b1101_1110]
        );
    }

    #[test]
    fn utf16_bytes_back_to_units() {
        for byte_order in [ByteOrder::Little, ByteOrder::Big] {
            let utf16_bytes = utf16_units_to_bytes(&SMILE_UTF16, byte_order);
            assert_eq!(
                bytes_to_utf16_units(&utf16_bytes, byte_order),
                Ok(SMILE_UTF16.to_vec()),
                "{byte_order:?}"
            );
        }
    }

    #[test]
    fn utf16_wrong_byte_order_gives_wrong_unit() {
        // H written Little, read Big → 0100_1000_0000_0000 = 䠀, not H
        let utf16_bytes = utf16_units_to_bytes(&[0b0000_0000_0100_1000], ByteOrder::Little);
        assert_eq!(
            bytes_to_utf16_units(&utf16_bytes, ByteOrder::Big),
            Ok(vec![0b0100_1000_0000_0000])
        );
    }

    #[test]
    fn utf16_odd_byte_count_is_an_error() {
        assert_eq!(
            bytes_to_utf16_units(&[0b1101_1000, 0b0011_1101, 0b1101_1110], ByteOrder::Big),
            Err(DecodeError::UnexpectedEnd)
        );
    }

    // ---------- UTF-32 ----------

    #[test]
    fn utf32_big_puts_left_byte_first() {
        assert_eq!(
            utf32_units_to_bytes(&SMILE_UTF32, ByteOrder::Big),
            [0b0000_0000, 0b0000_0001, 0b1111_0110, 0b0000_0000]
        );
    }

    #[test]
    fn utf32_little_puts_right_byte_first() {
        assert_eq!(
            utf32_units_to_bytes(&SMILE_UTF32, ByteOrder::Little),
            [0b0000_0000, 0b1111_0110, 0b0000_0001, 0b0000_0000]
        );
    }

    #[test]
    fn utf32_bytes_back_to_units() {
        for byte_order in [ByteOrder::Little, ByteOrder::Big] {
            let utf32_bytes = utf32_units_to_bytes(&SMILE_UTF32, byte_order);
            assert_eq!(
                bytes_to_utf32_units(&utf32_bytes, byte_order),
                Ok(SMILE_UTF32.to_vec()),
                "{byte_order:?}"
            );
        }
    }

    #[test]
    fn utf32_byte_count_not_multiple_of_4_is_an_error() {
        assert_eq!(
            bytes_to_utf32_units(&[0b0000_0000, 0b0000_0001, 0b1111_0110], ByteOrder::Big),
            Err(DecodeError::UnexpectedEnd)
        );
    }
}
