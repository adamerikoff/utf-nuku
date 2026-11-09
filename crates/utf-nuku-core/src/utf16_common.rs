//! Everything UTF-16 knows about units and surrogates, in one place.
//! Encode and decode only call these functions.
//!
//! Words used everywhere:
//! - unit                = 16 bits (u16)
//! - single unit         = a char that fits in 16 bits: the unit IS the code point
//! - surrogate pair      = 2 units for a char above 0xFFFF
//! - high surrogate      = the 1st unit of a pair: header 110110 + 10 data bits
//! - low surrogate       = the 2nd unit of a pair: header 110111 + 10 data bits
//! - lead unit           = the first unit of a char (a single unit or a high surrogate)
//! - unit_sequence_count = how many units the whole char has (1 or 2)

use crate::bits::{and, low_mask, or, shift_left, shift_right};
use crate::errors::DecodeError;

// ---------- unit sequence count ----------

/// How many units UTF-16 needs to store `code_point`: 1 or 2.
pub(crate) fn units_needed(code_point: u32) -> usize {
    match code_point {
        // fits in 16 bits → single unit
        0b0000_0000_0000_0000..=0b1111_1111_1111_1111 => 1,
        // 0x10000 ..= 0x10FFFF → surrogate pair
        0b0001_0000_0000_0000_0000..=0b0001_0000_1111_1111_1111_1111 => 2,
        // a `char` can never be above 0x10FFFF
        _ => unreachable!("code point {code_point:#b} is above 0x10FFFF"),
    }
}

// ---------- surrogate pair data: 20 bits split into 10 | 10 ----------

/// Subtract 0x10000: a code point above 0xFFFF becomes exactly 20 bits.
///
/// ```text
/// 😀 0b0001_1111_0110_0000_0000 - 0b0001_0000_0000_0000_0000 = 0b0000_1111_0110_0000_0000
/// ```
pub(crate) fn subtract_surrogate_offset(code_point: u32) -> u32 {
    code_point - 0b0001_0000_0000_0000_0000
}

/// Turn a code point above 0xFFFF into the data of a surrogate pair.
/// 1. subtract 0x10000 → the result always fits in exactly 20 bits
/// 2. split 10 | 10    → high surrogate data, low surrogate data
///
/// ```text
/// 😀 = 0b0001_1111_0110_0000_0000
/// 1. - 0b0001_0000_0000_0000_0000 = 0b0000_1111_0110_0000_0000   (20 bits)
/// 2. high = shift_right(…, 10)     = 0b00_0011_1101
///    low  = and(…, 0b11_1111_1111) = 0b10_0000_0000
/// ```
pub(crate) fn split_surrogate_pair(code_point: u32) -> (u32, u32) {
    // 1. 0x10000 ..= 0x10FFFF becomes 0x0 ..= 0xFFFFF (20 bits)
    let surrogate_pair_data = subtract_surrogate_offset(code_point);

    // 2. left 10 bits → high, right 10 bits → low
    let high_surrogate_data = shift_right(surrogate_pair_data, 10);
    let low_surrogate_data = and(surrogate_pair_data, low_mask(10));

    (high_surrogate_data, low_surrogate_data)
}

/// The opposite of `split_surrogate_pair`.
/// 1. glue 10 | 10 back into 20 bits
/// 2. add 0x10000
///
/// ```text
/// high 0b00_0011_1101, low 0b10_0000_0000
/// 1. or(shift_left(high, 10), low) = 0b0000_1111_0110_0000_0000
/// 2. + 0b0001_0000_0000_0000_0000  = 0b0001_1111_0110_0000_0000 = 😀
/// ```
pub(crate) fn join_surrogate_pair(high_surrogate_data: u32, low_surrogate_data: u32) -> u32 {
    // 1. make room for 10 bits, put the low data in
    let high_in_front = shift_left(high_surrogate_data, 10);
    let surrogate_pair_data = or(high_in_front, low_surrogate_data);

    // 2. undo the subtraction
    surrogate_pair_data + 0b0001_0000_0000_0000_0000
}

// ---------- high surrogate: 110110 + 10 data bits ----------

/// Build a high surrogate: header 110110 in front, 10 data bits behind it.
///
/// ```text
/// shift_left(110110, 10)                      = 1101_1000_0000_0000
/// or(1101_1000_0000_0000, 00_0011_1101)       = 1101_1000_0011_1101
/// ```
pub(crate) fn build_high_surrogate(high_surrogate_data: u32) -> u16 {
    let header_in_front = shift_left(0b11_0110, 10);
    let high_surrogate_bits = or(header_in_front, high_surrogate_data);

    high_surrogate_bits as u16
}

// ---------- low surrogate: 110111 + 10 data bits ----------

/// Build a low surrogate: header 110111 in front, 10 data bits behind it.
///
/// ```text
/// shift_left(110111, 10)                      = 1101_1100_0000_0000
/// or(1101_1100_0000_0000, 10_0000_0000)       = 1101_1110_0000_0000
/// ```
pub(crate) fn build_low_surrogate(low_surrogate_data: u32) -> u16 {
    let header_in_front = shift_left(0b11_0111, 10);
    let low_surrogate_bits = or(header_in_front, low_surrogate_data);

    low_surrogate_bits as u16
}

/// Read a low surrogate: it must be 110111xx_xxxxxxxx, then erase the header.
/// Returns the 10 low surrogate data bits, or an error if it isn't a low surrogate.
pub(crate) fn read_low_surrogate(low_surrogate: u16) -> Result<u32, DecodeError> {
    match low_surrogate {
        // 110111xx_xxxxxxxx
        0b1101_1100_0000_0000..=0b1101_1111_1111_1111 => {
            // erase the header, only the 10 data bits are left
            let low_surrogate_bits = u32::from(low_surrogate);
            let low_surrogate_data = and(low_surrogate_bits, low_mask(10));
            Ok(low_surrogate_data)
        }
        _ => Err(DecodeError::ExpectedLowSurrogate),
    }
}

// ---------- lead unit: the first unit of a char ----------

/// Read a lead unit: its range tells how many units the whole char has.
/// Returns `(unit_sequence_count, lead_unit_data)`:
/// - single unit    → (1, the code point itself)
/// - high surrogate → (2, its 10 data bits)
/// - low surrogate  → error: it can't start a char
pub(crate) fn read_lead_unit(lead_unit: u16) -> Result<(usize, u32), DecodeError> {
    let lead_unit_bits = u32::from(lead_unit);

    match lead_unit {
        // 110110xx_xxxxxxxx → high surrogate, a low surrogate must follow
        0b1101_1000_0000_0000..=0b1101_1011_1111_1111 => {
            // erase the header, only the 10 data bits are left
            let high_surrogate_data = and(lead_unit_bits, low_mask(10));
            Ok((2, high_surrogate_data))
        }
        // 110111xx_xxxxxxxx → low surrogate with no high surrogate before it
        0b1101_1100_0000_0000..=0b1101_1111_1111_1111 => Err(DecodeError::UnpairedLowSurrogate),
        // anything else → single unit, it IS the code point
        _ => Ok((1, lead_unit_bits)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- units_needed ----------

    #[test]
    fn units_needed_edges() {
        assert_eq!(units_needed(0b0100_1000), 1, "H");
        assert_eq!(units_needed(0b0010_0000_1010_1100), 1, "€");
        assert_eq!(
            units_needed(0b1111_1111_1111_1111),
            1,
            "0xFFFF: last single unit"
        );
        assert_eq!(
            units_needed(0b0001_0000_0000_0000_0000),
            2,
            "0x10000: first pair"
        );
        assert_eq!(units_needed(0b0001_1111_0110_0000_0000), 2, "😀");
    }

    // ---------- split / join surrogate pair ----------

    #[test]
    fn split_surrogate_pair_examples() {
        // 😀
        assert_eq!(
            split_surrogate_pair(0b0001_1111_0110_0000_0000),
            (0b00_0011_1101, 0b10_0000_0000)
        );
        // 0x10000: smallest → all data bits 0
        assert_eq!(split_surrogate_pair(0b0001_0000_0000_0000_0000), (0, 0));
        // 0x10FFFF: largest → all data bits 1
        assert_eq!(
            split_surrogate_pair(0b0001_0000_1111_1111_1111_1111),
            (0b11_1111_1111, 0b11_1111_1111)
        );
    }

    #[test]
    fn join_is_the_opposite_of_split() {
        assert_eq!(
            join_surrogate_pair(0b00_0011_1101, 0b10_0000_0000),
            0b0001_1111_0110_0000_0000
        );
        assert_eq!(join_surrogate_pair(0, 0), 0b0001_0000_0000_0000_0000);
    }

    // ---------- build ----------

    #[test]
    fn build_surrogates_add_headers() {
        assert_eq!(build_high_surrogate(0b00_0011_1101), 0b1101_1000_0011_1101);
        assert_eq!(build_low_surrogate(0b10_0000_0000), 0b1101_1110_0000_0000);
    }

    // ---------- read ----------

    #[test]
    fn read_lead_unit_kinds() {
        // single unit: € → the code point itself
        assert_eq!(
            read_lead_unit(0b0010_0000_1010_1100),
            Ok((1, 0b0010_0000_1010_1100))
        );
        // high surrogate of 😀 → 10 data bits
        assert_eq!(
            read_lead_unit(0b1101_1000_0011_1101),
            Ok((2, 0b00_0011_1101))
        );
        // low surrogate can't start a char
        assert_eq!(
            read_lead_unit(0b1101_1110_0000_0000),
            Err(DecodeError::UnpairedLowSurrogate)
        );
    }

    #[test]
    fn read_low_surrogate_checks() {
        assert_eq!(
            read_low_surrogate(0b1101_1110_0000_0000),
            Ok(0b10_0000_0000)
        );
        // a normal unit is not a low surrogate
        assert_eq!(
            read_low_surrogate(0b0100_1000),
            Err(DecodeError::ExpectedLowSurrogate)
        );
        // a high surrogate is not a low surrogate
        assert_eq!(
            read_low_surrogate(0b1101_1000_0011_1101),
            Err(DecodeError::ExpectedLowSurrogate)
        );
    }
}
