//! UTF-16 decoding: 16-bit units → string. The mirror of utf16_encode.
//!
//! `decode_char` decodes ONE char and records every step in a `Utf16CharTrace`:
//! 1. `read_lead_unit`      — single unit (1) or high surrogate (2)? + its data
//! 2. `read_low_surrogate`  — only for pairs: check 110111, keep its 10 data bits
//! 3. `join_surrogate_pair` — only for pairs: glue 10 | 10, add 0x10000
//! 4. `code_point_to_char`  — Unicode rules: is it a real char?
//!
//! `trace_decode_utf16` = one trace per char, until the first error.
//! `decode_utf16`       = only the chars from those traces, or the error.

use crate::errors::DecodeError;
use crate::unicode::code_point_to_char;
use crate::utf16_common::{
    join_surrogate_pair, read_lead_unit, read_low_surrogate, subtract_surrogate_offset,
};
use crate::utf16_trace::{SurrogatePairTrace, Utf16CharTrace};

/// Decode ONE char from the start of `remaining_units` (its lead unit is at position 0),
/// keeping every step. `unit_sequence_count` in the trace tells the caller where the next char starts.
///
/// ```text
/// [1101_1000_0011_1101, 1101_1110_0000_0000, 0000_0000_0100_1000]  →  trace of '😀', unit_sequence_count 2
/// ```
pub(crate) fn decode_char(remaining_units: &[u16]) -> Result<Utf16CharTrace, DecodeError> {
    // 1. lead unit
    let lead_unit = remaining_units[0];
    let (unit_sequence_count, lead_unit_data) = read_lead_unit(lead_unit)?;

    // input cut off after a high surrogate (e.g. [1101_1000_0011_1101] alone)
    if unit_sequence_count > remaining_units.len() {
        return Err(DecodeError::UnexpectedEnd);
    }

    let code_point;
    let surrogate_pair;

    if unit_sequence_count == 1 {
        // single unit: its data IS the code point, no pair steps
        code_point = lead_unit_data;
        surrogate_pair = None;
    } else {
        // 2. the low surrogate at position 1
        let high_surrogate = lead_unit;
        let high_surrogate_data = lead_unit_data;
        let low_surrogate = remaining_units[1];
        let low_surrogate_data = read_low_surrogate(low_surrogate)?;

        // 3. glue high + low, add 0x10000
        code_point = join_surrogate_pair(high_surrogate_data, low_surrogate_data);

        // the 20-bit value in between (only kept for the trace)
        let surrogate_pair_data = subtract_surrogate_offset(code_point);

        surrogate_pair = Some(SurrogatePairTrace {
            surrogate_pair_data,
            high_surrogate_data,
            low_surrogate_data,
            high_surrogate,
            low_surrogate,
        });
    }

    // 4. number → char (Unicode rules)
    let character = code_point_to_char(code_point)?;

    // all units of the char: a copy of this char's part of the input
    let character_units = remaining_units[..unit_sequence_count].to_vec();

    Ok(Utf16CharTrace {
        character,
        code_point,
        unit_sequence_count,
        surrogate_pair,
        character_units,
    })
}

/// Decode UTF-16 units, one trace per char.
/// Stops at the first error: returns the traces of the chars before it + the error.
/// `None` = no error, everything was decoded.
pub fn trace_decode_utf16(utf16_units: &[u16]) -> (Vec<Utf16CharTrace>, Option<DecodeError>) {
    let mut char_traces = Vec::new();
    // position of the current char's lead unit
    let mut lead_unit_index = 0;

    while lead_unit_index < utf16_units.len() {
        // all units from this char's lead unit to the end (a view, nothing is copied)
        let remaining_units = &utf16_units[lead_unit_index..];

        match decode_char(remaining_units) {
            Ok(char_trace) => {
                // jump over this char's units to the next lead unit
                lead_unit_index += char_trace.unit_sequence_count;
                char_traces.push(char_trace);
            }
            Err(error) => return (char_traces, Some(error)),
        }
    }

    (char_traces, None)
}

/// Decode UTF-16 units into a String, or stop at the first error.
pub fn decode_utf16(utf16_units: &[u16]) -> Result<String, DecodeError> {
    let (char_traces, decode_error) = trace_decode_utf16(utf16_units);
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
    use crate::utf16_encode::encode_char;

    /// decode_char, but only the char and how many units it used
    fn decode_char_and_count(remaining_units: &[u16]) -> Result<(char, usize), DecodeError> {
        let char_trace = decode_char(remaining_units)?;
        Ok((char_trace.character, char_trace.unit_sequence_count))
    }

    #[test]
    fn decode_trace_equals_encode_trace() {
        // same steps, opposite direction → the same trace
        for character in [
            'H',
            'é',
            '€',
            '😀',
            '\0',
            '\u{FFFF}',
            '\u{10000}',
            '\u{10FFFF}',
        ] {
            let encode_trace = encode_char(character);
            let decode_trace = decode_char(&encode_trace.character_units);
            assert_eq!(decode_trace, Ok(encode_trace), "{character:?}");
        }
    }

    #[test]
    fn trace_keeps_chars_before_the_error() {
        // "H", then a lone low surrogate
        let (char_traces, decode_error) =
            trace_decode_utf16(&[0b0000_0000_0100_1000, 0b1101_1110_0000_0000]);
        assert_eq!(char_traces.len(), 1);
        assert_eq!(char_traces[0].character, 'H');
        assert_eq!(decode_error, Some(DecodeError::UnpairedLowSurrogate));
    }

    // ---------- decode_char ----------

    #[test]
    fn decode_char_single_unit() {
        assert_eq!(
            decode_char_and_count(&[0b0010_0000_1010_1100]),
            Ok(('€', 1))
        );
    }

    #[test]
    fn decode_char_surrogate_pair() {
        assert_eq!(
            decode_char_and_count(&[0b1101_1000_0011_1101, 0b1101_1110_0000_0000]),
            Ok(('😀', 2))
        );
    }

    #[test]
    fn decode_char_stops_at_its_own_units() {
        // 😀 is 2 units; the 3rd unit (H) belongs to the next char
        assert_eq!(
            decode_char_and_count(&[
                0b1101_1000_0011_1101,
                0b1101_1110_0000_0000,
                0b0000_0000_0100_1000
            ]),
            Ok(('😀', 2))
        );
    }

    // ---------- decode_utf16: valid input ----------

    #[test]
    fn decode_known_units() {
        assert_eq!(
            decode_utf16(&[
                0b0000_0000_0100_1000, // H
                0b0000_0000_1110_1001, // é
                0b0010_0000_1010_1100, // €
                0b1101_1000_0011_1101, // 😀 high
                0b1101_1110_0000_0000, // 😀 low
            ]),
            Ok("Hé€😀".to_string())
        );
    }

    #[test]
    fn decode_matches_rust() {
        for text in ["Hé€😀🎉", "", "\0", "\u{FFFF}", "\u{10000}", "\u{10FFFF}"] {
            let utf16_units: Vec<u16> = text.encode_utf16().collect();
            assert_eq!(decode_utf16(&utf16_units), Ok(text.to_string()), "{text:?}");
        }
    }

    // ---------- decode_utf16: errors ----------

    #[test]
    fn error_low_surrogate_first() {
        assert_eq!(
            decode_utf16(&[0b1101_1110_0000_0000]),
            Err(DecodeError::UnpairedLowSurrogate)
        );
    }

    #[test]
    fn error_high_surrogate_at_end() {
        // high surrogate promises 2 units, only 1 given
        assert_eq!(
            decode_utf16(&[0b1101_1000_0011_1101]),
            Err(DecodeError::UnexpectedEnd)
        );
    }

    #[test]
    fn error_high_surrogate_then_normal_unit() {
        // high surrogate, then H instead of a low surrogate
        assert_eq!(
            decode_utf16(&[0b1101_1000_0011_1101, 0b0000_0000_0100_1000]),
            Err(DecodeError::ExpectedLowSurrogate)
        );
    }

    #[test]
    fn error_two_high_surrogates() {
        assert_eq!(
            decode_utf16(&[0b1101_1000_0011_1101, 0b1101_1000_0011_1101]),
            Err(DecodeError::ExpectedLowSurrogate)
        );
    }
}
