//! The steps of one UTF-16 char.
//!
//! ```text
//! single unit (€):                 surrogate pair (😀):
//! code point    0010_0000_1010_1100   code point      0_0001_1111_0110_0000_0000
//! units needed  1                     units needed    2
//! unit          0010000010101100      − 0x10000       0000_1111_0110_0000_0000
//! bytes         20 ac                 split           0000111101  1000000000
//!                                     high surrogate  [110110][0000111101] = d83d
//!                                     low surrogate   [110111][1000000000] = de00
//!                                     bytes           d8 3d de 00
//! ```

use dioxus::prelude::*;
use utf_nuku_core::{Utf16CharTrace, bits_text};

use crate::step_parts::{DataBits, HeaderDataBits, StepRow, hex_text, plain_bits};

#[component]
pub fn Utf16Steps(char_trace: Utf16CharTrace, character_bytes: Vec<u8>) -> Element {
    let bytes_hex = hex_text(&character_bytes);

    match char_trace.surrogate_pair {
        // single unit: the code point itself, no header
        None => {
            let code_point_text = bits_text(char_trace.code_point, 16);
            let unit_text = plain_bits(char_trace.code_point, 16);
            rsx! {
                StepRow { label: "code point",
                    DataBits { data_text: code_point_text }
                }
                StepRow { label: "units needed", "1" }
                StepRow { label: "unit",
                    DataBits { data_text: unit_text }
                }
                StepRow { label: "bytes", span { class: "hex", "{bytes_hex}" } }
            }
        }
        // surrogate pair: − 0x10000, split 10 | 10, add headers
        Some(surrogate_pair) => {
            let code_point_text = bits_text(char_trace.code_point, 21);
            let surrogate_pair_data_text = bits_text(surrogate_pair.surrogate_pair_data, 20);
            let high_surrogate_data_text = plain_bits(surrogate_pair.high_surrogate_data, 10);
            let low_surrogate_data_text = plain_bits(surrogate_pair.low_surrogate_data, 10);
            let high_surrogate_hex = format!("{:04x}", surrogate_pair.high_surrogate);
            let low_surrogate_hex = format!("{:04x}", surrogate_pair.low_surrogate);

            rsx! {
                StepRow { label: "code point",
                    DataBits { data_text: code_point_text }
                }
                StepRow { label: "units needed", "2" }
                StepRow { label: "− 0x10000",
                    DataBits { data_text: surrogate_pair_data_text }
                }
                StepRow { label: "split 10 | 10",
                    DataBits { data_text: high_surrogate_data_text.clone() }
                    " "
                    DataBits { data_text: low_surrogate_data_text.clone() }
                }
                StepRow { label: "high surrogate",
                    HeaderDataBits { header_text: "110110", data_text: high_surrogate_data_text }
                    span { class: "hex", " = {high_surrogate_hex}" }
                }
                StepRow { label: "low surrogate",
                    HeaderDataBits { header_text: "110111", data_text: low_surrogate_data_text }
                    span { class: "hex", " = {low_surrogate_hex}" }
                }
                StepRow { label: "bytes", span { class: "hex", "{bytes_hex}" } }
            }
        }
    }
}
