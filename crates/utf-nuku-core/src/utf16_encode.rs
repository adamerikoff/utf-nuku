//! UTF-16 encoding: string → 16-bit units.
//!
//! `encode_char` encodes ONE char and records every step in a `Utf16CharTrace`:
//! - fits in 16 bits → 1 unit, the code point itself
//! - above 0xFFFF    → 2 units:
//!   1. `subtract_surrogate_offset` — subtract 0x10000 → 20 bits
//!   2. `split_surrogate_pair`      — split 20 bits into 10 | 10
//!   3. `build_high_surrogate`      — 110110 + high 10 bits
//!   4. `build_low_surrogate`       — 110111 + low 10 bits
//!
//! `trace_encode_utf16` = one trace per char. `encode_utf16` = only the units from those traces.

use crate::utf16_common::{
    build_high_surrogate, build_low_surrogate, split_surrogate_pair, subtract_surrogate_offset,
    units_needed,
};
use crate::utf16_trace::{SurrogatePairTrace, Utf16CharTrace};

/// Encode a char above 0xFFFF into a surrogate pair, keeping every step.
fn encode_surrogate_pair(code_point: u32) -> SurrogatePairTrace {
    // 1. subtract 0x10000 → 20 bits (only kept for the trace)
    let surrogate_pair_data = subtract_surrogate_offset(code_point);

    // 2. split 10 | 10 (split_surrogate_pair does step 1 again inside)
    let (high_surrogate_data, low_surrogate_data) = split_surrogate_pair(code_point);

    // 3. + 4. add the headers
    let high_surrogate = build_high_surrogate(high_surrogate_data);
    let low_surrogate = build_low_surrogate(low_surrogate_data);

    SurrogatePairTrace {
        surrogate_pair_data,
        high_surrogate_data,
        low_surrogate_data,
        high_surrogate,
        low_surrogate,
    }
}

/// Encode one char into UTF-16, keeping every step.
pub(crate) fn encode_char(character: char) -> Utf16CharTrace {
    let code_point = u32::from(character);
    let unit_sequence_count = units_needed(code_point);

    let surrogate_pair;
    let mut character_units = Vec::new();

    if unit_sequence_count == 1 {
        // fits in 16 bits: the unit is the code point itself, no pair steps
        surrogate_pair = None;
        let single_unit = code_point as u16;
        character_units.push(single_unit);
    } else {
        let surrogate_pair_trace = encode_surrogate_pair(code_point);
        character_units.push(surrogate_pair_trace.high_surrogate);
        character_units.push(surrogate_pair_trace.low_surrogate);
        surrogate_pair = Some(surrogate_pair_trace);
    }

    Utf16CharTrace {
        character,
        code_point,
        unit_sequence_count,
        surrogate_pair,
        character_units,
    }
}

/// Encode a whole string, one trace per char.
pub fn trace_encode_utf16(text: &str) -> Vec<Utf16CharTrace> {
    let mut char_traces = Vec::new();
    for character in text.chars() {
        let char_trace = encode_char(character);
        char_traces.push(char_trace);
    }
    char_traces
}

/// Encode a whole string into UTF-16 units: every char's units in one flat list.
pub fn encode_utf16(text: &str) -> Vec<u16> {
    let mut utf16_units = Vec::new();
    for char_trace in trace_encode_utf16(text) {
        utf16_units.extend(char_trace.character_units);
    }
    utf16_units
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- every step ----------

    #[test]
    fn encode_char_single_unit_steps() {
        assert_eq!(
            encode_char('€'),
            Utf16CharTrace {
                character: '€',
                code_point: 0b0010_0000_1010_1100,
                unit_sequence_count: 1,
                surrogate_pair: None,
                character_units: vec![0b0010_0000_1010_1100],
            }
        );
    }

    #[test]
    fn encode_char_surrogate_pair_steps() {
        assert_eq!(
            encode_char('😀'),
            Utf16CharTrace {
                character: '😀',
                code_point: 0b0001_1111_0110_0000_0000,
                unit_sequence_count: 2,
                surrogate_pair: Some(SurrogatePairTrace {
                    surrogate_pair_data: 0b0000_1111_0110_0000_0000,
                    high_surrogate_data: 0b00_0011_1101,
                    low_surrogate_data: 0b10_0000_0000,
                    high_surrogate: 0b1101_1000_0011_1101,
                    low_surrogate: 0b1101_1110_0000_0000,
                }),
                character_units: vec![0b1101_1000_0011_1101, 0b1101_1110_0000_0000],
            }
        );
    }

    // ---------- final units ----------

    #[test]
    fn encode_char_single_units() {
        assert_eq!(
            encode_char('H').character_units,
            [0b0000_0000_0100_1000],
            "H"
        );
        assert_eq!(
            encode_char('é').character_units,
            [0b0000_0000_1110_1001],
            "é"
        );
    }

    #[test]
    fn encode_char_surrogate_pairs() {
        // 🎉 = 0x1F389 → D83C DF89
        assert_eq!(
            encode_char('🎉').character_units,
            [0b1101_1000_0011_1100, 0b1101_1111_1000_1001],
            "🎉"
        );
    }

    #[test]
    fn encode_matches_rust() {
        // Rust's own UTF-16 encoder is the reference.
        for text in ["Hé€😀", "", "\0", "\u{FFFF}", "\u{10000}", "\u{10FFFF}"] {
            let expected: Vec<u16> = text.encode_utf16().collect();
            assert_eq!(encode_utf16(text), expected, "{text:?}");
        }
    }
}
