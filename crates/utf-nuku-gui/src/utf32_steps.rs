//! The steps of one UTF-32 char: the unit IS the code point.

use dioxus::prelude::*;
use utf_nuku_core::{Utf32CharTrace, bits_text};

use crate::step_parts::{DataBits, StepRow, hex_text};

#[component]
pub fn Utf32Steps(char_trace: Utf32CharTrace, character_bytes: Vec<u8>) -> Element {
    let code_point_text = bits_text(char_trace.code_point, 21);
    let unit_text = bits_text(char_trace.unit, 32);
    let bytes_hex = hex_text(&character_bytes);

    rsx! {
        StepRow { label: "code point",
            DataBits { data_text: code_point_text }
        }
        StepRow { label: "unit",
            DataBits { data_text: unit_text }
        }
        StepRow { label: "bytes", span { class: "hex", "{bytes_hex}" } }
    }
}
