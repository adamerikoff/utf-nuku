//! UTF-8 encoding: string → bytes.
//!
//! `encode_char` encodes ONE char and records every step in a `Utf8CharTrace`:
//! 1. `bytes_needed`            — how many bytes does this char need?
//! 2. `split`                   — cut the code point: lead byte data + 6-bit continuation bytes data
//! 3. `build_lead_byte`         — header + the lead byte data
//! 4. `build_continuation_byte` — 10 + one continuation byte data, for each
//!
//! `trace_encode_utf8` = one trace per char. `encode_utf8` = only the bytes from those traces.
//! One code path: the bytes and the trace can never disagree.

use crate::bits::split;
use crate::utf8_common::{
    build_continuation_byte, build_lead_byte, bytes_needed, lead_byte_header,
};
use crate::utf8_trace::Utf8CharTrace;

/// Encode one char into UTF-8, keeping every step.
pub(crate) fn encode_char(character: char) -> Utf8CharTrace {
    let code_point = u32::from(character);

    // 1. how many bytes
    let byte_sequence_count = bytes_needed(code_point);

    // 2. 6 data bits per continuation byte (cut from the right);
    //    what's left is the lead byte data
    let continuation_byte_count = byte_sequence_count - 1;
    let (lead_byte_data, continuation_bytes_data) = split(code_point, continuation_byte_count, 6);

    // 3. lead byte
    let (lead_byte_header, lead_byte_header_bit_count) = lead_byte_header(byte_sequence_count);
    let lead_byte = build_lead_byte(byte_sequence_count, lead_byte_data);

    // 4. continuation bytes
    let mut continuation_bytes = Vec::new();
    for &continuation_byte_data in &continuation_bytes_data {
        let continuation_byte = build_continuation_byte(continuation_byte_data);
        continuation_bytes.push(continuation_byte);
    }

    // all bytes of the char: lead byte, then continuation bytes
    let mut character_bytes = vec![lead_byte];
    character_bytes.extend(&continuation_bytes);

    Utf8CharTrace {
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
    }
}

/// Encode a whole string, one trace per char.
pub fn trace_encode_utf8(text: &str) -> Vec<Utf8CharTrace> {
    let mut char_traces = Vec::new();
    for character in text.chars() {
        let char_trace = encode_char(character);
        char_traces.push(char_trace);
    }
    char_traces
}

/// Encode a whole string into UTF-8 bytes: every char's bytes in one flat list.
pub fn encode_utf8(text: &str) -> Vec<u8> {
    let mut utf8_bytes = Vec::new();
    for char_trace in trace_encode_utf8(text) {
        // extend = append all of this char's bytes to the end of `utf8_bytes`
        utf8_bytes.extend(char_trace.character_bytes);
    }
    utf8_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- every step ----------

    #[test]
    fn encode_char_one_byte_steps() {
        assert_eq!(
            encode_char('H'),
            Utf8CharTrace {
                character: 'H',
                code_point: 0b0100_1000,
                byte_sequence_count: 1,
                lead_byte_header: 0b0,
                lead_byte_header_bit_count: 1,
                lead_byte_data: 0b0100_1000,
                continuation_bytes_data: vec![],
                lead_byte: 0b0100_1000,
                continuation_bytes: vec![],
                character_bytes: vec![0b0100_1000],
            }
        );
    }

    #[test]
    fn encode_char_two_byte_steps() {
        assert_eq!(
            encode_char('é'),
            Utf8CharTrace {
                character: 'é',
                code_point: 0b1110_1001,
                byte_sequence_count: 2,
                lead_byte_header: 0b110,
                lead_byte_header_bit_count: 3,
                lead_byte_data: 0b0_0011,
                continuation_bytes_data: vec![0b10_1001],
                lead_byte: 0b1100_0011,
                continuation_bytes: vec![0b1010_1001],
                character_bytes: vec![0b1100_0011, 0b1010_1001],
            }
        );
    }

    #[test]
    fn encode_char_four_byte_steps() {
        // 😀 = 000 | 011111 | 011000 | 000000
        assert_eq!(
            encode_char('😀'),
            Utf8CharTrace {
                character: '😀',
                code_point: 0b0001_1111_0110_0000_0000,
                byte_sequence_count: 4,
                lead_byte_header: 0b1_1110,
                lead_byte_header_bit_count: 5,
                lead_byte_data: 0b000,
                continuation_bytes_data: vec![0b01_1111, 0b01_1000, 0b00_0000],
                lead_byte: 0b1111_0000,
                continuation_bytes: vec![0b1001_1111, 0b1001_1000, 0b1000_0000],
                character_bytes: vec![0b1111_0000, 0b1001_1111, 0b1001_1000, 0b1000_0000],
            }
        );
    }

    // ---------- final bytes ----------

    #[test]
    fn encode_char_known_bytes() {
        assert_eq!(
            encode_char('€').character_bytes,
            [0b1110_0010, 0b1000_0010, 0b1010_1100]
        );
    }

    #[test]
    fn encode_char_matches_rust() {
        // Rust's own encoder is the reference.
        for character in [
            'H',
            'é',
            '€',
            '😀',
            '\0',
            '\u{7F}',
            '\u{80}',
            '\u{FFFF}',
            '\u{10FFFF}',
        ] {
            let mut buffer = [0u8; 4];
            let expected = character.encode_utf8(&mut buffer).as_bytes();
            assert_eq!(
                encode_char(character).character_bytes,
                expected,
                "char U+{:04X}",
                character as u32
            );
        }
    }

    #[test]
    fn encode_string_matches_rust() {
        let text = "Hé€😀";
        assert_eq!(encode_utf8(text), text.as_bytes(), "{text}");
    }

    #[test]
    fn encode_empty_string() {
        assert!(encode_utf8("").is_empty(), "empty string → no bytes");
        assert!(trace_encode_utf8("").is_empty(), "empty string → no traces");
    }
}
