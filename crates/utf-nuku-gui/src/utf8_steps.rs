//! The steps of one UTF-8 char.
//!
//! ```text
//! code point      000_1110_1001
//! bytes needed    2
//! split           00011  101001
//! lead byte       [110][00011]   = c3
//! continuation    [10][101001]   = a9
//! ```

use dioxus::prelude::*;
use utf_nuku_core::{Utf8CharTrace, bits_text, utf8_code_point_bit_count};

use crate::step_parts::{DataBits, HeaderDataBits, StepRow, hex_text, plain_bits};

#[component]
pub fn Utf8Steps(char_trace: Utf8CharTrace) -> Element {
    let code_point_bit_count = utf8_code_point_bit_count(
        char_trace.lead_byte_header_bit_count,
        char_trace.byte_sequence_count,
    );
    let lead_byte_data_bit_count = 8 - char_trace.lead_byte_header_bit_count;

    let code_point_text = bits_text(char_trace.code_point, code_point_bit_count);
    let lead_byte_header_text = plain_bits(
        char_trace.lead_byte_header,
        char_trace.lead_byte_header_bit_count,
    );
    let lead_byte_data_text = plain_bits(char_trace.lead_byte_data, lead_byte_data_bit_count);
    let lead_byte_hex = hex_text(&[char_trace.lead_byte]);

    // one entry per continuation byte: (data bits, byte in hex)
    let mut continuation_texts = Vec::new();
    for continuation_index in 0..char_trace.continuation_bytes.len() {
        let continuation_byte_data = char_trace.continuation_bytes_data[continuation_index];
        let continuation_byte = char_trace.continuation_bytes[continuation_index];
        let continuation_data_text = plain_bits(continuation_byte_data, 6);
        let continuation_hex = hex_text(&[continuation_byte]);
        continuation_texts.push((continuation_data_text, continuation_hex));
    }

    rsx! {
        StepRow { label: "code point",
            DataBits { data_text: code_point_text }
        }
        StepRow { label: "bytes needed", "{char_trace.byte_sequence_count}" }
        StepRow { label: "split",
            DataBits { data_text: lead_byte_data_text.clone() }
            for (continuation_data_text, _) in continuation_texts.clone() {
                " "
                DataBits { data_text: continuation_data_text }
            }
        }
        StepRow { label: "lead byte",
            HeaderDataBits { header_text: lead_byte_header_text, data_text: lead_byte_data_text }
            span { class: "hex", " = {lead_byte_hex}" }
        }
        for (continuation_data_text, continuation_hex) in continuation_texts {
            StepRow { label: "continuation",
                HeaderDataBits { header_text: "10", data_text: continuation_data_text }
                span { class: "hex", " = {continuation_hex}" }
            }
        }
    }
}
