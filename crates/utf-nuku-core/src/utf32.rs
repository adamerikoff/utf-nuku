//! UTF-32: every char is exactly ONE 32-bit unit, and that unit IS the code point.
//! No splitting, no headers, no overlong (there's only one way to write each number).
//!
//! encode: char → code point → unit
//! decode: unit → Unicode rules (unicode::code_point_to_char) → char
//!
//! Like UTF-8/16, every char is recorded in a trace (`Utf32CharTrace`),
//! and `encode_utf32` / `decode_utf32` only take the units / chars from those traces.

use crate::errors::DecodeError;
use crate::unicode::code_point_to_char;
use crate::utf32_trace::Utf32CharTrace;

// ---------- encode ----------

/// Encode one char into UTF-32, keeping every step.
///
/// ```text
/// 'é' → code_point 1110_1001 → unit 0000_0000_0000_0000_0000_0000_1110_1001
/// ```
pub(crate) fn encode_char(character: char) -> Utf32CharTrace {
    let code_point = u32::from(character);
    // the unit is the code point itself
    let unit = code_point;

    Utf32CharTrace {
        character,
        code_point,
        unit,
    }
}

/// Encode a whole string, one trace per char.
pub fn trace_encode_utf32(text: &str) -> Vec<Utf32CharTrace> {
    let mut char_traces = Vec::new();
    for character in text.chars() {
        let char_trace = encode_char(character);
        char_traces.push(char_trace);
    }
    char_traces
}

/// Encode a whole string into UTF-32 units: one unit per char.
pub fn encode_utf32(text: &str) -> Vec<u32> {
    let mut utf32_units = Vec::new();
    for char_trace in trace_encode_utf32(text) {
        utf32_units.push(char_trace.unit);
    }
    utf32_units
}

// ---------- decode ----------

/// Decode one unit, keeping every step.
/// The unit must pass the Unicode rules (not above 0x10FFFF, not a surrogate).
pub(crate) fn decode_char(unit: u32) -> Result<Utf32CharTrace, DecodeError> {
    let code_point = unit;
    let character = code_point_to_char(code_point)?;

    Ok(Utf32CharTrace {
        character,
        code_point,
        unit,
    })
}

/// Decode UTF-32 units, one trace per char.
/// Stops at the first error: returns the traces of the chars before it + the error.
/// `None` = no error, everything was decoded.
pub fn trace_decode_utf32(utf32_units: &[u32]) -> (Vec<Utf32CharTrace>, Option<DecodeError>) {
    let mut char_traces = Vec::new();
    for &unit in utf32_units {
        match decode_char(unit) {
            Ok(char_trace) => char_traces.push(char_trace),
            Err(error) => return (char_traces, Some(error)),
        }
    }
    (char_traces, None)
}

/// Decode UTF-32 units into a String, or stop at the first error.
pub fn decode_utf32(utf32_units: &[u32]) -> Result<String, DecodeError> {
    let (char_traces, decode_error) = trace_decode_utf32(utf32_units);
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

    #[test]
    fn encode_char_steps() {
        assert_eq!(
            encode_char('😀'),
            Utf32CharTrace {
                character: '😀',
                code_point: 0b0001_1111_0110_0000_0000,
                unit: 0b0001_1111_0110_0000_0000,
            }
        );
    }

    #[test]
    fn decode_trace_equals_encode_trace() {
        for character in ['H', 'é', '€', '😀', '\0', '\u{10FFFF}'] {
            let encode_trace = encode_char(character);
            assert_eq!(
                decode_char(encode_trace.unit),
                Ok(encode_trace),
                "{character:?}"
            );
        }
    }

    #[test]
    fn encode_known_units() {
        assert_eq!(
            encode_utf32("Hé€😀"),
            [
                0b0100_1000,
                0b1110_1001,
                0b0010_0000_1010_1100,
                0b0001_1111_0110_0000_0000,
            ]
        );
    }

    #[test]
    fn encode_empty() {
        assert!(encode_utf32("").is_empty());
    }

    #[test]
    fn decode_known_units() {
        assert_eq!(
            decode_utf32(&[
                0b0100_1000,
                0b1110_1001,
                0b0010_0000_1010_1100,
                0b0001_1111_0110_0000_0000,
            ]),
            Ok("Hé€😀".to_string())
        );
    }

    #[test]
    fn round_trip() {
        let text = "Hé€😀\0\u{10FFFF}";
        assert_eq!(decode_utf32(&encode_utf32(text)), Ok(text.to_string()));
    }

    #[test]
    fn trace_keeps_chars_before_the_error() {
        // "H", then a surrogate
        let (char_traces, decode_error) = trace_decode_utf32(&[0b0100_1000, 0b1101_1000_0000_0000]);
        assert_eq!(char_traces.len(), 1);
        assert_eq!(char_traces[0].character, 'H');
        assert_eq!(decode_error, Some(DecodeError::InvalidCodePoint));
    }

    #[test]
    fn error_too_big() {
        // 0x110000
        assert_eq!(
            decode_utf32(&[0b0001_0001_0000_0000_0000_0000]),
            Err(DecodeError::InvalidCodePoint)
        );
    }

    #[test]
    fn error_surrogate() {
        // 0xD800
        assert_eq!(
            decode_utf32(&[0b1101_1000_0000_0000]),
            Err(DecodeError::InvalidCodePoint)
        );
    }
}
