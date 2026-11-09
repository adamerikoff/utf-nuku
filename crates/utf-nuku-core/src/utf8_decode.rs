//! UTF-8 decoding: bytes → string. The mirror of utf8_encode.
//!
//! `decode_char` decodes ONE char and records every step in a `Utf8CharTrace`:
//! 1. `read_lead_byte`         — how many bytes is this char? + the lead byte data
//! 2. `read_continuation_byte` — for each next byte: check 10, keep its 6 data bits
//! 3. `join`                   — glue lead byte data + continuation bytes data into one code point
//! 4. `code_point_to_char`     — Unicode rules: is it a real char?
//! 5. `is_overlong`            — UTF-8 rule: was the shortest encoding used?
//!
//! `trace_decode_utf8` = one trace per char, until the first error.
//! `decode_utf8`       = only the chars from those traces, or the error.

use crate::bits::join;
use crate::errors::DecodeError;
use crate::unicode::code_point_to_char;
use crate::utf8_common::{is_overlong, lead_byte_header, read_continuation_byte, read_lead_byte};
use crate::utf8_trace::Utf8CharTrace;

/// Decode ONE char from the start of `remaining_bytes` (its lead byte is at position 0),
/// keeping every step. `byte_sequence_count` in the trace tells the caller where the next char starts.
///
/// ```text
/// [1100_0011, 1010_1001, 0110_0001]  →  trace of 'é', byte_sequence_count 2
///                                       (0110_0001 belongs to the next char)
/// ```
pub(crate) fn decode_char(remaining_bytes: &[u8]) -> Result<Utf8CharTrace, DecodeError> {
    // 1. lead byte
    let lead_byte = remaining_bytes[0];
    // `?` = if it returned an error, stop and return that error from decode_char
    let (byte_sequence_count, lead_byte_data) = read_lead_byte(lead_byte)?;
    let (lead_byte_header, lead_byte_header_bit_count) = lead_byte_header(byte_sequence_count);

    // input cut off in the middle of a char (e.g. [1100_0011] alone)
    if byte_sequence_count > remaining_bytes.len() {
        return Err(DecodeError::UnexpectedEnd);
    }

    // 2. continuation bytes: positions 1 .. byte_sequence_count
    let mut continuation_bytes = Vec::new();
    let mut continuation_bytes_data = Vec::new();
    for continuation_byte_index in 1..byte_sequence_count {
        let continuation_byte = remaining_bytes[continuation_byte_index];
        let continuation_byte_data = read_continuation_byte(continuation_byte)?;
        continuation_bytes.push(continuation_byte);
        continuation_bytes_data.push(continuation_byte_data);
    }

    // 3. glue the data bits back together
    let code_point = join(lead_byte_data, continuation_bytes_data.clone(), 6);

    // 4. number → char (Unicode rules: not too big, not a surrogate)
    let character = code_point_to_char(code_point)?;

    // 5. UTF-8 rule: shortest encoding only
    if is_overlong(code_point, byte_sequence_count) {
        return Err(DecodeError::Overlong);
    }

    // all bytes of the char: a copy of this char's part of the input
    let character_bytes = remaining_bytes[..byte_sequence_count].to_vec();

    Ok(Utf8CharTrace {
        character,
        code_point,
        byte_sequence_count,
        lead_byte_header,
        lead_byte_header_bit_count,
        lead_byte_data,
        continuation_bytes_data,
        lead_byte,
        continuation_bytes,
        character_bytes,
    })
}

/// Decode UTF-8 bytes, one trace per char.
/// Stops at the first error: returns the traces of the chars before it + the error.
/// `None` = no error, everything was decoded.
pub fn trace_decode_utf8(utf8_bytes: &[u8]) -> (Vec<Utf8CharTrace>, Option<DecodeError>) {
    let mut char_traces = Vec::new();
    // position of the current char's lead byte
    let mut lead_byte_index = 0;

    while lead_byte_index < utf8_bytes.len() {
        // all bytes from this char's lead byte to the end (a view, nothing is copied)
        let remaining_bytes = &utf8_bytes[lead_byte_index..];

        match decode_char(remaining_bytes) {
            Ok(char_trace) => {
                // jump over this char's bytes to the next lead byte
                lead_byte_index += char_trace.byte_sequence_count;
                char_traces.push(char_trace);
            }
            Err(error) => return (char_traces, Some(error)),
        }
    }

    (char_traces, None)
}

/// Decode UTF-8 bytes into a String, or stop at the first error.
pub fn decode_utf8(utf8_bytes: &[u8]) -> Result<String, DecodeError> {
    let (char_traces, decode_error) = trace_decode_utf8(utf8_bytes);
    if let Some(error) = decode_error {
        return Err(error);
    }

    let mut decoded_text = String::new();
    for char_trace in char_traces {
        decoded_text.push(char_trace.character);
    }
    Ok(decoded_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf8_encode::encode_char;

    /// decode_char, but only the char and how many bytes it used
    fn decode_char_and_count(remaining_bytes: &[u8]) -> Result<(char, usize), DecodeError> {
        let char_trace = decode_char(remaining_bytes)?;
        Ok((char_trace.character, char_trace.byte_sequence_count))
    }

    // ---------- decode_char ----------

    #[test]
    fn decode_trace_equals_encode_trace() {
        // same steps, opposite direction → the same trace
        for character in ['H', 'é', '€', '😀', '\0', '\u{10FFFF}'] {
            let encode_trace = encode_char(character);
            let decode_trace = decode_char(&encode_trace.character_bytes);
            assert_eq!(decode_trace, Ok(encode_trace), "{character:?}");
        }
    }

    #[test]
    fn decode_char_one_per_byte_sequence_count() {
        assert_eq!(decode_char_and_count(&[0b0100_1000]), Ok(('H', 1)));
        assert_eq!(
            decode_char_and_count(&[0b1100_0011, 0b1010_1001]),
            Ok(('é', 2))
        );
        assert_eq!(
            decode_char_and_count(&[0b1110_0010, 0b1000_0010, 0b1010_1100]),
            Ok(('€', 3))
        );
        assert_eq!(
            decode_char_and_count(&[0b1111_0000, 0b1001_1111, 0b1001_1000, 0b1000_0000]),
            Ok(('😀', 4))
        );
    }

    #[test]
    fn decode_char_stops_at_its_own_bytes() {
        // é is 2 bytes; the 3rd byte (a) belongs to the next char and is not touched
        assert_eq!(
            decode_char_and_count(&[0b1100_0011, 0b1010_1001, 0b0110_0001]),
            Ok(('é', 2))
        );
    }

    // ---------- decode_utf8: valid input ----------

    #[test]
    fn decode_known_bytes() {
        assert_eq!(decode_utf8(&[0b0100_1000]), Ok("H".to_string()));
        assert_eq!(
            decode_utf8(&[0b1100_0011, 0b1010_1001]),
            Ok("é".to_string())
        );
        assert_eq!(
            decode_utf8(&[0b1110_0010, 0b1000_0010, 0b1010_1100]),
            Ok("€".to_string())
        );
        assert_eq!(
            decode_utf8(&[0b1111_0000, 0b1001_1111, 0b1001_1000, 0b1000_0000]),
            Ok("😀".to_string())
        );
    }

    #[test]
    fn decode_matches_rust() {
        let text = "Hé€😀";
        assert_eq!(decode_utf8(text.as_bytes()), Ok(text.to_string()));
    }

    #[test]
    fn decode_empty() {
        assert_eq!(decode_utf8(&[]), Ok(String::new()));
    }

    // ---------- trace_decode_utf8 ----------

    #[test]
    fn trace_keeps_chars_before_the_error() {
        // "H", then a lone continuation byte
        let (char_traces, decode_error) = trace_decode_utf8(&[0b0100_1000, 0b1000_0000]);
        assert_eq!(char_traces.len(), 1);
        assert_eq!(char_traces[0].character, 'H');
        assert_eq!(decode_error, Some(DecodeError::InvalidStartByte));
    }

    #[test]
    fn decode_round_trip_edges() {
        for character in [
            '\0',
            '\u{7F}',
            '\u{80}',
            '\u{7FF}',
            '\u{800}',
            '\u{FFFF}',
            '\u{10000}',
            '\u{10FFFF}',
        ] {
            let text = character.to_string();
            assert_eq!(
                decode_utf8(text.as_bytes()),
                Ok(text.clone()),
                "U+{:04X}",
                character as u32
            );
        }
    }

    // ---------- decode_utf8: errors ----------

    #[test]
    fn error_continuation_at_start() {
        assert_eq!(
            decode_utf8(&[0b1000_0000]),
            Err(DecodeError::InvalidStartByte)
        );
    }

    #[test]
    fn error_invalid_byte() {
        assert_eq!(
            decode_utf8(&[0b1111_1111]),
            Err(DecodeError::InvalidStartByte)
        );
    }

    #[test]
    fn error_unexpected_end() {
        // 110xxxxx promises 2 bytes, only 1 given
        assert_eq!(decode_utf8(&[0b1100_0011]), Err(DecodeError::UnexpectedEnd));
    }

    #[test]
    fn error_expected_continuation() {
        // 110xxxxx then 'A' (0xxxxxxx) instead of 10xxxxxx
        assert_eq!(
            decode_utf8(&[0b1100_0011, 0b0100_0001]),
            Err(DecodeError::ExpectedContinuation)
        );
    }

    #[test]
    fn error_overlong() {
        // '/' (0b010_1111) forced into 2 bytes: 110_00000 10_101111
        assert_eq!(
            decode_utf8(&[0b1100_0000, 0b1010_1111]),
            Err(DecodeError::Overlong)
        );
    }

    #[test]
    fn error_surrogate() {
        // 0xD800 (a UTF-16 surrogate) written as 3 bytes: not a valid char
        assert_eq!(
            decode_utf8(&[0b1110_1101, 0b1010_0000, 0b1000_0000]),
            Err(DecodeError::InvalidCodePoint)
        );
    }

    #[test]
    fn error_above_max() {
        // 0x110000: one above the largest code point
        assert_eq!(
            decode_utf8(&[0b1111_0100, 0b1001_0000, 0b1000_0000, 0b1000_0000]),
            Err(DecodeError::InvalidCodePoint)
        );
    }

    #[test]
    fn error_in_second_char() {
        // "H" is fine, then a lone continuation byte at index 1
        assert_eq!(
            decode_utf8(&[0b0100_1000, 0b1000_0000]),
            Err(DecodeError::InvalidStartByte)
        );
    }
}
