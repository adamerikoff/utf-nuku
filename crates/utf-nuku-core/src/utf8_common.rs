//! Everything UTF-8 knows about bytes and headers, in one place.
//! Encode and decode only call these functions.
//!
//! Words used everywhere:
//! - lead byte           = the first byte of a char (header 0 / 110 / 1110 / 11110)
//! - continuation byte   = every byte after it      (header 10, then 6 data bits)
//! - byte data           = the data bits of a byte, without its header
//! - byte_sequence_count = how many bytes the whole char has (1–4)

use crate::bits::{and, low_mask, or, shift_left};
use crate::errors::DecodeError;

// ---------- byte sequence count ----------

/// How many bytes UTF-8 needs to store `code_point` (the char's number): 1, 2, 3 or 4.
pub(crate) fn bytes_needed(code_point: u32) -> usize {
    match code_point {
        // up to 7 bits
        0b0000_0000..=0b0111_1111 => 1,
        // up to 11 bits
        0b1000_0000..=0b0111_1111_1111 => 2,
        // up to 16 bits
        0b1000_0000_0000..=0b1111_1111_1111_1111 => 3,
        // up to 0x10FFFF, the largest code point Unicode has
        0b0001_0000_0000_0000_0000..=0b0001_0000_1111_1111_1111_1111 => 4,
        // a `char` can never be above 0x10FFFF
        _ => unreachable!("code point {code_point:#b} is above 0x10FFFF"),
    }
}

// ---------- lead byte ----------

/// THE header table. Both building and reading a lead byte use it.
/// Returns `(header, header_bit_count)` for a char of `byte_sequence_count` bytes.
pub(crate) fn lead_byte_header(byte_sequence_count: usize) -> (u32, u32) {
    match byte_sequence_count {
        1 => (0b0, 1),      // 0xxxxxxx
        2 => (0b110, 3),    // 110xxxxx
        3 => (0b1110, 4),   // 1110xxxx
        4 => (0b1_1110, 5), // 11110xxx
        // bytes_needed only returns 1–4
        _ => unreachable!("UTF-8 char can't be {byte_sequence_count} bytes"),
    }
}

/// Build a lead byte: header in front, the lead byte data behind it.
///
/// ```text
/// 2 bytes, lead byte data 0_0011:
///   shift_left(110, 8 - 3)    = 1100_0000
///   or(1100_0000, 0000_0011)  = 1100_0011
/// ```
pub(crate) fn build_lead_byte(byte_sequence_count: usize, lead_byte_data: u32) -> u8 {
    let (header, header_bit_count) = lead_byte_header(byte_sequence_count);
    let data_bit_count = 8 - header_bit_count;

    let header_in_front = shift_left(header, data_bit_count);
    let lead_byte_bits = or(header_in_front, lead_byte_data);

    lead_byte_bits as u8
}

/// Read a lead byte: its header tells how many bytes the whole char has, then erase the header.
/// Returns `(byte_sequence_count, lead_byte_data)`, or an error if no char can start with this byte.
///
/// ```text
/// 1100_0011:
///   in 1100_0000..=1101_1111 (110xxxxx) → the char is 2 bytes long
///   header 110 is 3 bits → 5 data bits → and(1100_0011, 0001_1111) = 0000_0011
/// ```
pub(crate) fn read_lead_byte(lead_byte: u8) -> Result<(usize, u32), DecodeError> {
    // 1. which header? → how many bytes the whole char has
    let byte_sequence_count = match lead_byte {
        0b0000_0000..=0b0111_1111 => 1, // 0xxxxxxx → lead only
        0b1100_0000..=0b1101_1111 => 2, // 110xxxxx → lead + 1 continuation
        0b1110_0000..=0b1110_1111 => 3, // 1110xxxx → lead + 2 continuations
        0b1111_0000..=0b1111_0111 => 4, // 11110xxx → lead + 3 continuations
        // 10xxxxxx (continuation) or 11111xxx: a char can't start here
        _ => return Err(DecodeError::InvalidStartByte),
    };

    // 2. erase the header, only the data bits are left
    let (_header, header_bit_count) = lead_byte_header(byte_sequence_count);
    let data_bit_count = 8 - header_bit_count;
    let lead_byte_data_mask = low_mask(data_bit_count);
    let lead_byte_bits = u32::from(lead_byte);
    let lead_byte_data = and(lead_byte_bits, lead_byte_data_mask);

    Ok((byte_sequence_count, lead_byte_data))
}

// ---------- continuation byte: 10xxxxxx ----------

/// Build a continuation byte: 10 in front, the 6-bit continuation byte data behind it.
///
/// ```text
/// shift_left(10, 6)          = 1000_0000
/// or(1000_0000, 0010_1001)   = 1010_1001
/// ```
pub(crate) fn build_continuation_byte(continuation_byte_data: u32) -> u8 {
    let header_in_front = shift_left(0b10, 6);
    let continuation_byte_bits = or(header_in_front, continuation_byte_data);

    continuation_byte_bits as u8
}

/// Read a continuation byte: it must be 10xxxxxx, then erase the header.
/// Returns the 6-bit continuation byte data, or an error if it isn't a continuation byte.
///
/// ```text
/// 1010_1001:
///   in 1000_0000..=1011_1111 (10xxxxxx) ✅
///   and(1010_1001, 0011_1111) = 0010_1001
/// ```
pub(crate) fn read_continuation_byte(continuation_byte: u8) -> Result<u32, DecodeError> {
    match continuation_byte {
        // 10xxxxxx
        0b1000_0000..=0b1011_1111 => {
            // erase the header, only the 6 data bits are left
            let continuation_byte_bits = u32::from(continuation_byte);
            let continuation_byte_data = and(continuation_byte_bits, low_mask(6));
            Ok(continuation_byte_data)
        }
        // 0xxxxxxx or 11xxxxxx: not a continuation byte
        _ => Err(DecodeError::ExpectedContinuation),
    }
}

// ---------- overlong ----------

/// UTF-8 rule: every char must use the SHORTEST encoding.
/// Overlong = more bytes were used than this number needs ('/' as 2 bytes).
/// Call it only after unicode::code_point_to_char passed (bytes_needed can't take > 0x10FFFF).
pub(crate) fn is_overlong(code_point: u32, byte_sequence_count: usize) -> bool {
    let shortest_byte_sequence_count = bytes_needed(code_point);
    byte_sequence_count != shortest_byte_sequence_count
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- bytes_needed ----------

    #[test]
    fn bytes_needed_one_char_per_range() {
        assert_eq!(bytes_needed('H' as u32), 1, "H");
        assert_eq!(bytes_needed('é' as u32), 2, "é");
        assert_eq!(bytes_needed('€' as u32), 3, "€");
        assert_eq!(bytes_needed('😀' as u32), 4, "😀");
    }

    #[test]
    fn bytes_needed_range_edges() {
        // last value of one range, first value of the next
        assert_eq!(bytes_needed(0b0111_1111), 1, "last 1-byte");
        assert_eq!(bytes_needed(0b1000_0000), 2, "first 2-byte");
        assert_eq!(bytes_needed(0b0111_1111_1111), 2, "last 2-byte");
        assert_eq!(bytes_needed(0b1000_0000_0000), 3, "first 3-byte");
        assert_eq!(bytes_needed(0b1111_1111_1111_1111), 3, "last 3-byte");
        assert_eq!(bytes_needed(0b0001_0000_0000_0000_0000), 4, "first 4-byte");
        assert_eq!(
            bytes_needed(0b0001_0000_1111_1111_1111_1111),
            4,
            "last code point"
        );
    }

    // ---------- lead byte ----------

    #[test]
    fn build_lead_byte_per_byte_sequence_count() {
        assert_eq!(build_lead_byte(1, 0b100_1000), 0b0100_1000, "H");
        assert_eq!(build_lead_byte(2, 0b0_0011), 0b1100_0011, "é");
        assert_eq!(build_lead_byte(3, 0b0010), 0b1110_0010, "€");
        assert_eq!(build_lead_byte(4, 0b000), 0b1111_0000, "😀");
    }

    #[test]
    fn read_lead_byte_per_byte_sequence_count() {
        assert_eq!(read_lead_byte(0b0100_1000), Ok((1, 0b100_1000)), "H");
        assert_eq!(read_lead_byte(0b1100_0011), Ok((2, 0b0_0011)), "é");
        assert_eq!(read_lead_byte(0b1110_0010), Ok((3, 0b0010)), "€");
        assert_eq!(read_lead_byte(0b1111_0000), Ok((4, 0b000)), "😀");
    }

    #[test]
    fn read_lead_byte_rejects() {
        assert_eq!(
            read_lead_byte(0b1010_1001),
            Err(DecodeError::InvalidStartByte),
            "10xxxxxx can't start a char"
        );
        assert_eq!(
            read_lead_byte(0b1111_1000),
            Err(DecodeError::InvalidStartByte),
            "11111xxx is never valid"
        );
        assert_eq!(
            read_lead_byte(0b1111_1111),
            Err(DecodeError::InvalidStartByte),
            "11111111 is never valid"
        );
    }

    // ---------- continuation byte ----------

    #[test]
    fn build_continuation_byte_adds_10() {
        assert_eq!(build_continuation_byte(0b10_1001), 0b1010_1001);
        assert_eq!(build_continuation_byte(0b00_0000), 0b1000_0000);
        assert_eq!(build_continuation_byte(0b11_1111), 0b1011_1111);
    }

    #[test]
    fn read_continuation_byte_keeps_6_bits() {
        assert_eq!(read_continuation_byte(0b1010_1001), Ok(0b10_1001));
        assert_eq!(
            read_continuation_byte(0b0100_0001),
            Err(DecodeError::ExpectedContinuation),
            "0xxxxxxx is not 10xxxxxx"
        );
        assert_eq!(
            read_continuation_byte(0b1100_0011),
            Err(DecodeError::ExpectedContinuation),
            "110xxxxx is not 10xxxxxx"
        );
    }

    // ---------- is_overlong ----------

    #[test]
    fn is_overlong_checks() {
        // é in 2 bytes: shortest → fine
        assert!(!is_overlong(0b000_1110_1001, 2), "é as 2 bytes");
        // '/' in 2 bytes: needs only 1 → overlong
        assert!(is_overlong(0b010_1111, 2), "'/' as 2 bytes");
        // 'A' in 3 bytes: needs only 1 → overlong
        assert!(is_overlong(0b100_0001, 3), "'A' as 3 bytes");
    }
}
