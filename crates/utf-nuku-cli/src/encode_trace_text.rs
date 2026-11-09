//! -v for encode: core's traces → text, in encode order (char → bytes).
//!
//! ```text
//! 'é'
//!   code point      000_1110_1001
//!   bytes needed    2
//!   lead byte       110 + 0_0011 → 1100_0011
//!   continuation    10 + 10_1001 → 1010_1001
//! ```

use utf_nuku_core::{
    Encoding, Utf8CharTrace, Utf16CharTrace, Utf32CharTrace, bits_text, trace_encode_utf8,
    trace_encode_utf16, trace_encode_utf32, utf8_code_point_bit_count,
};

use crate::trace_layout::row;

// ---------- one char ----------

fn utf8_char_text(char_trace: &Utf8CharTrace) -> String {
    let code_point_bit_count = utf8_code_point_bit_count(
        char_trace.lead_byte_header_bit_count,
        char_trace.byte_sequence_count,
    );
    let lead_byte_data_bit_count = 8 - char_trace.lead_byte_header_bit_count;

    let mut lines = Vec::new();
    lines.push(format!("{:?}", char_trace.character));
    lines.push(row(
        "code point",
        &bits_text(char_trace.code_point, code_point_bit_count),
    ));
    lines.push(row(
        "bytes needed",
        &char_trace.byte_sequence_count.to_string(),
    ));

    // lead byte: header + data → byte
    let lead_byte_header_text = bits_text(
        char_trace.lead_byte_header,
        char_trace.lead_byte_header_bit_count,
    );
    let lead_byte_data_text = bits_text(char_trace.lead_byte_data, lead_byte_data_bit_count);
    let lead_byte_text = bits_text(u32::from(char_trace.lead_byte), 8);
    lines.push(row(
        "lead byte",
        &format!("{lead_byte_header_text} + {lead_byte_data_text} → {lead_byte_text}"),
    ));

    // continuation bytes: 10 + 6 data bits → byte
    for continuation_index in 0..char_trace.continuation_bytes.len() {
        let continuation_byte_data = char_trace.continuation_bytes_data[continuation_index];
        let continuation_byte = char_trace.continuation_bytes[continuation_index];
        let continuation_byte_data_text = bits_text(continuation_byte_data, 6);
        let continuation_byte_text = bits_text(u32::from(continuation_byte), 8);
        lines.push(row(
            "continuation",
            &format!("10 + {continuation_byte_data_text} → {continuation_byte_text}"),
        ));
    }

    lines.join("\n")
}

fn utf16_char_text(char_trace: &Utf16CharTrace) -> String {
    let mut lines = Vec::new();
    lines.push(format!("{:?}", char_trace.character));

    match &char_trace.surrogate_pair {
        // single unit: the code point itself
        None => {
            let unit_text = bits_text(char_trace.code_point, 16);
            lines.push(row("code point", &unit_text));
            lines.push(row("units needed", "1"));
            lines.push(row("unit", &unit_text));
        }
        // surrogate pair: − 0x10000, split 10 | 10, add headers
        Some(surrogate_pair) => {
            let high_surrogate_data_text = bits_text(surrogate_pair.high_surrogate_data, 10);
            let low_surrogate_data_text = bits_text(surrogate_pair.low_surrogate_data, 10);
            let high_surrogate_text = bits_text(u32::from(surrogate_pair.high_surrogate), 16);
            let low_surrogate_text = bits_text(u32::from(surrogate_pair.low_surrogate), 16);

            lines.push(row("code point", &bits_text(char_trace.code_point, 21)));
            lines.push(row("units needed", "2"));
            lines.push(row(
                "− 0x10000",
                &bits_text(surrogate_pair.surrogate_pair_data, 20),
            ));
            // headers: 110110 = high, 110111 = low
            lines.push(row(
                "high surrogate",
                &format!("110110 + {high_surrogate_data_text} → {high_surrogate_text}"),
            ));
            lines.push(row(
                "low surrogate",
                &format!("110111 + {low_surrogate_data_text} → {low_surrogate_text}"),
            ));
        }
    }

    lines.join("\n")
}

fn utf32_char_text(char_trace: &Utf32CharTrace) -> String {
    let lines = vec![
        format!("{:?}", char_trace.character),
        row("code point", &bits_text(char_trace.code_point, 21)),
        row("unit", &bits_text(char_trace.unit, 32)),
    ];
    lines.join("\n")
}

// ---------- whole text ----------

/// Every char of `text`, one block per char, blank line between blocks.
pub fn encode_trace_text(text: &str, encoding: Encoding) -> String {
    let mut char_texts = Vec::new();

    match encoding {
        Encoding::Utf8 => {
            for char_trace in trace_encode_utf8(text) {
                char_texts.push(utf8_char_text(&char_trace));
            }
        }
        Encoding::Utf16 => {
            for char_trace in trace_encode_utf16(text) {
                char_texts.push(utf16_char_text(&char_trace));
            }
        }
        Encoding::Utf32 => {
            for char_trace in trace_encode_utf32(text) {
                char_texts.push(utf32_char_text(&char_trace));
            }
        }
    }

    char_texts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_e_acute() {
        let expected_lines = [
            "'é'",
            "  code point      000_1110_1001",
            "  bytes needed    2",
            "  lead byte       110 + 0_0011 → 1100_0011",
            "  continuation    10 + 10_1001 → 1010_1001",
        ];
        assert_eq!(
            encode_trace_text("é", Encoding::Utf8),
            expected_lines.join("\n")
        );
    }

    #[test]
    fn utf16_smile() {
        let expected_lines = [
            "'😀'",
            "  code point      0_0001_1111_0110_0000_0000",
            "  units needed    2",
            "  − 0x10000       0000_1111_0110_0000_0000",
            "  high surrogate  110110 + 00_0011_1101 → 1101_1000_0011_1101",
            "  low surrogate   110111 + 10_0000_0000 → 1101_1110_0000_0000",
        ];
        assert_eq!(
            encode_trace_text("😀", Encoding::Utf16),
            expected_lines.join("\n")
        );
    }
}
