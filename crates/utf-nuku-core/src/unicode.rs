//! Unicode rules that every encoding shares (UTF-8, UTF-16, UTF-32).
//! They are about the code point itself, not about bytes or units.

use crate::errors::DecodeError;

/// Check a decoded number against the 2 Unicode rules, then turn it into a char.
pub(crate) fn code_point_to_char(code_point: u32) -> Result<char, DecodeError> {
    // Rule 1: Unicode stops at 0x10FFFF
    let is_too_big = code_point > 0b0001_0000_1111_1111_1111_1111;
    if is_too_big {
        return Err(DecodeError::InvalidCodePoint);
    }

    // Rule 2: 0xD800..=0xDFFF is reserved for UTF-16 surrogates, no chars live there
    let is_surrogate = code_point >= 0b1101_1000_0000_0000 && code_point <= 0b1101_1111_1111_1111;
    if is_surrogate {
        return Err(DecodeError::InvalidCodePoint);
    }

    // Both rules passed. from_u32 returns Some(char) for every number that passes them;
    // `expect` takes the char out of the Some (it would only panic if our rules were wrong).
    let character = char::from_u32(code_point).expect("rules 1 and 2 were checked above");
    Ok(character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_code_points() {
        assert_eq!(code_point_to_char(0b0100_1000), Ok('H'));
        assert_eq!(code_point_to_char(0b000_1110_1001), Ok('é'));
        assert_eq!(code_point_to_char(0b0001_1111_0110_0000_0000), Ok('😀'));
        // last code point
        assert_eq!(
            code_point_to_char(0b0001_0000_1111_1111_1111_1111),
            Ok('\u{10FFFF}')
        );
    }

    #[test]
    fn rule_1_too_big() {
        // 0x110000: one above the last code point
        assert_eq!(
            code_point_to_char(0b0001_0001_0000_0000_0000_0000),
            Err(DecodeError::InvalidCodePoint)
        );
    }

    #[test]
    fn rule_2_surrogates() {
        // first and last surrogate: 0xD800, 0xDFFF
        assert_eq!(
            code_point_to_char(0b1101_1000_0000_0000),
            Err(DecodeError::InvalidCodePoint)
        );
        assert_eq!(
            code_point_to_char(0b1101_1111_1111_1111),
            Err(DecodeError::InvalidCodePoint)
        );
        // just outside the range: 0xD7FF and 0xE000 are real chars
        assert!(code_point_to_char(0b1101_0111_1111_1111).is_ok());
        assert!(code_point_to_char(0b1110_0000_0000_0000).is_ok());
    }
}
