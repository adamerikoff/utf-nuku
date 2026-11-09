//! -v for decode: core's traces → text, in decode order (bytes → char).
//! Shows the chars decoded before an error; the error itself is printed by main.rs.
//!
//! ```text
//! 'é'
//!   lead byte       1100_0011 → 110 + 0_0011
//!   bytes needed    2
//!   continuation    1010_1001 → 10 + 10_1001
//!   code point      000_1110_1001
//! ```

use utf_nuku_core::{
    ByteOrder, DecodeError, Encoding, Utf8CharTrace, Utf16CharTrace, Utf32CharTrace, bits_text,
    bytes_to_utf16_units, bytes_to_utf32_units, trace_decode_utf8, trace_decode_utf16,
    trace_decode_utf32, utf8_code_point_bit_count,
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

    // lead byte → header + data
    let lead_byte_text = bits_text(u32::from(char_trace.lead_byte), 8);
    let lead_byte_header_text = bits_text(
        char_trace.lead_byte_header,
        char_trace.lead_byte_header_bit_count,
    );
    let lead_byte_data_text = bits_text(char_trace.lead_byte_data, lead_byte_data_bit_count);
    lines.push(row(
        "lead byte",
        &format!("{lead_byte_text} → {lead_byte_header_text} + {lead_byte_data_text}"),
    ));
    lines.push(row(
        "bytes needed",
        &char_trace.byte_sequence_count.to_string(),
    ));

    // continuation bytes → 10 + 6 data bits
    for continuation_index in 0..char_trace.continuation_bytes.len() {
        let continuation_byte = char_trace.continuation_bytes[continuation_index];
        let continuation_byte_data = char_trace.continuation_bytes_data[continuation_index];
        let continuation_byte_text = bits_text(u32::from(continuation_byte), 8);
        let continuation_byte_data_text = bits_text(continuation_byte_data, 6);
        lines.push(row(
            "continuation",
            &format!("{continuation_byte_text} → 10 + {continuation_byte_data_text}"),
        ));
    }

    // joined data = code point
    lines.push(row(
        "code point",
        &bits_text(char_trace.code_point, code_point_bit_count),
    ));

    lines.join("\n")
}

fn utf16_char_text(char_trace: &Utf16CharTrace) -> String {
    let mut lines = Vec::new();
    lines.push(format!("{:?}", char_trace.character));

    match &char_trace.surrogate_pair {
        // single unit: it IS the code point
        None => {
            let unit_text = bits_text(char_trace.code_point, 16);
            lines.push(row("unit", &unit_text));
            lines.push(row("units needed", "1"));
            lines.push(row("code point", &unit_text));
        }
        // surrogate pair: remove headers, join 10 | 10, + 0x10000
        Some(surrogate_pair) => {
            let high_surrogate_text = bits_text(u32::from(surrogate_pair.high_surrogate), 16);
            let low_surrogate_text = bits_text(u32::from(surrogate_pair.low_surrogate), 16);
            let high_surrogate_data_text = bits_text(surrogate_pair.high_surrogate_data, 10);
            let low_surrogate_data_text = bits_text(surrogate_pair.low_surrogate_data, 10);

            lines.push(row(
                "high surrogate",
                &format!("{high_surrogate_text} → 110110 + {high_surrogate_data_text}"),
            ));
            lines.push(row("units needed", "2"));
            lines.push(row(
                "low surrogate",
                &format!("{low_surrogate_text} → 110111 + {low_surrogate_data_text}"),
            ));
            lines.push(row(
                "joined 10 | 10",
                &bits_text(surrogate_pair.surrogate_pair_data, 20),
            ));
            lines.push(row("+ 0x10000", &bits_text(char_trace.code_point, 21)));
        }
    }

    lines.join("\n")
}

fn utf32_char_text(char_trace: &Utf32CharTrace) -> String {
    let lines = vec![
        format!("{:?}", char_trace.character),
        row("unit", &bits_text(char_trace.unit, 32)),
        row("code point", &bits_text(char_trace.code_point, 21)),
    ];
    lines.join("\n")
}

// ---------- whole input ----------

/// Every decoded char, one block per char, blank line between blocks.
/// Error only if the bytes can't even be grouped into units (leftover bytes).
pub fn decode_trace_text(
    encoded_bytes: &[u8],
    encoding: Encoding,
    byte_order: ByteOrder,
) -> Result<String, DecodeError> {
    let mut char_texts = Vec::new();

    // decode errors are ignored here: the traces stop at the error,
    // and run_decode reports the error itself
    match encoding {
        Encoding::Utf8 => {
            let (char_traces, _decode_error) = trace_decode_utf8(encoded_bytes);
            for char_trace in char_traces {
                char_texts.push(utf8_char_text(&char_trace));
            }
        }
        Encoding::Utf16 => {
            let utf16_units = bytes_to_utf16_units(encoded_bytes, byte_order)?;
            let (char_traces, _decode_error) = trace_decode_utf16(&utf16_units);
            for char_trace in char_traces {
                char_texts.push(utf16_char_text(&char_trace));
            }
        }
        Encoding::Utf32 => {
            let utf32_units = bytes_to_utf32_units(encoded_bytes, byte_order)?;
            let (char_traces, _decode_error) = trace_decode_utf32(&utf32_units);
            for char_trace in char_traces {
                char_texts.push(utf32_char_text(&char_trace));
            }
        }
    }

    Ok(char_texts.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_e_acute() {
        let expected_lines = [
            "'é'",
            "  lead byte       1100_0011 → 110 + 0_0011",
            "  bytes needed    2",
            "  continuation    1010_1001 → 10 + 10_1001",
            "  code point      000_1110_1001",
        ];
        assert_eq!(
            decode_trace_text(&[0b1100_0011, 0b1010_1001], Encoding::Utf8, ByteOrder::Big),
            Ok(expected_lines.join("\n"))
        );
    }

    #[test]
    fn stops_before_the_error() {
        // 'H', then a lone continuation byte: only 'H' is traced
        let trace_text =
            decode_trace_text(&[0b0100_1000, 0b1000_0000], Encoding::Utf8, ByteOrder::Big);
        assert_eq!(
            trace_text,
            Ok([
                "'H'",
                "  lead byte       0100_1000 → 0 + 100_1000",
                "  bytes needed    1",
                "  code point      100_1000"
            ]
            .join("\n"))
        );
    }
}
