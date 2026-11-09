//! One entry point for text ↔ bytes, for any encoding.
//! The CLI and GUI only deal with bytes (files, pipes, screen), so they call these 2 functions.
//!
//! encode = 2 steps (UTF-8 needs only 1):
//! ```text
//! UTF-8:   text → encode_utf8  → bytes
//! UTF-16:  text → encode_utf16 → u16 units → utf16_units_to_bytes → bytes
//! UTF-32:  text → encode_utf32 → u32 units → utf32_units_to_bytes → bytes
//! ```
//!
//! decode = the same 2 steps, reversed:
//! ```text
//! UTF-8:   bytes → decode_utf8 → text
//! UTF-16:  bytes → bytes_to_utf16_units → u16 units → decode_utf16 → text
//! UTF-32:  bytes → bytes_to_utf32_units → u32 units → decode_utf32 → text
//! ```
use crate::byte_order::ByteOrder;
use crate::byte_order::{
    bytes_to_utf16_units, bytes_to_utf32_units, utf16_units_to_bytes, utf32_units_to_bytes,
};
use crate::errors::DecodeError;
use crate::utf8_decode::decode_utf8;
use crate::utf8_encode::encode_utf8;
use crate::utf16_decode::decode_utf16;
use crate::utf16_encode::encode_utf16;
use crate::utf32::{decode_utf32, encode_utf32};

/// Which UTF to use.
/// `Clone, Copy` → copied like a number. `Debug, PartialEq, Eq` → for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16,
    Utf32,
}

/// Text → final bytes, ready for a file or pipe.
/// `byte_order` is ignored for UTF-8 (its unit is already 1 byte).
pub fn encode_to_bytes(text: &str, encoding: Encoding, byte_order: ByteOrder) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => encode_utf8(text),
        Encoding::Utf16 => {
            let utf16_units = encode_utf16(text);
            utf16_units_to_bytes(&utf16_units, byte_order)
        }
        Encoding::Utf32 => {
            let utf32_units = encode_utf32(text);
            utf32_units_to_bytes(&utf32_units, byte_order)
        }
    }
}

/// Bytes from a file or pipe → text, or the first error.
/// `byte_order` is ignored for UTF-8.
pub fn decode_from_bytes(
    bytes: &[u8],
    encoding: Encoding,
    byte_order: ByteOrder,
) -> Result<String, DecodeError> {
    match encoding {
        Encoding::Utf8 => decode_utf8(bytes),
        Encoding::Utf16 => {
            // ? → leftover bytes error goes straight to the caller
            let utf16_units = bytes_to_utf16_units(bytes, byte_order)?;
            decode_utf16(&utf16_units)
        }
        Encoding::Utf32 => {
            let utf32_units = bytes_to_utf32_units(bytes, byte_order)?;
            decode_utf32(&utf32_units)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- encode_to_bytes ----------

    #[test]
    fn encode_utf8_ignores_byte_order() {
        for byte_order in [ByteOrder::Little, ByteOrder::Big] {
            assert_eq!(
                encode_to_bytes("Hé", Encoding::Utf8, byte_order),
                [0b0100_1000, 0b1100_0011, 0b1010_1001],
                "{byte_order:?}"
            );
        }
    }

    #[test]
    fn encode_utf16_big() {
        assert_eq!(
            encode_to_bytes("😀", Encoding::Utf16, ByteOrder::Big),
            [0b1101_1000, 0b0011_1101, 0b1101_1110, 0b0000_0000]
        );
    }

    #[test]
    fn encode_utf32_little() {
        assert_eq!(
            encode_to_bytes("😀", Encoding::Utf32, ByteOrder::Little),
            [0b0000_0000, 0b1111_0110, 0b0000_0001, 0b0000_0000]
        );
    }

    // ---------- round trip: every encoding × both orders ----------

    #[test]
    fn encode_then_decode_gives_same_text() {
        let text = "Hé€😀\0\u{10FFFF}";
        for encoding in [Encoding::Utf8, Encoding::Utf16, Encoding::Utf32] {
            for byte_order in [ByteOrder::Little, ByteOrder::Big] {
                let encoded_bytes = encode_to_bytes(text, encoding, byte_order);
                assert_eq!(
                    decode_from_bytes(&encoded_bytes, encoding, byte_order),
                    Ok(text.to_string()),
                    "{encoding:?} {byte_order:?}"
                );
            }
        }
    }

    // ---------- decode errors ----------

    #[test]
    fn decode_utf16_leftover_byte_is_an_error() {
        assert_eq!(
            decode_from_bytes(
                &[0b1101_1000, 0b0011_1101, 0b1101_1110],
                Encoding::Utf16,
                ByteOrder::Big
            ),
            Err(DecodeError::UnexpectedEnd)
        );
    }
}
