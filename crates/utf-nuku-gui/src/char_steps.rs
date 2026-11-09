//! The steps card of the selected char: picks the view for its encoding.

use dioxus::prelude::*;
use utf_nuku_core::ByteOrder;

use crate::char_trace::CharTrace;
use crate::utf8_steps::Utf8Steps;
use crate::utf16_steps::Utf16Steps;
use crate::utf32_steps::Utf32Steps;

#[component]
pub fn CharSteps(char_trace: CharTrace, byte_order: ByteOrder) -> Element {
    let character_label = format!("{:?}", char_trace.character());
    let character_bytes = char_trace.character_bytes(byte_order);

    let encoding_steps = match char_trace {
        CharTrace::Utf8(utf8_trace) => rsx! {
            Utf8Steps { char_trace: utf8_trace }
        },
        CharTrace::Utf16(utf16_trace) => rsx! {
            Utf16Steps { char_trace: utf16_trace, character_bytes }
        },
        CharTrace::Utf32(utf32_trace) => rsx! {
            Utf32Steps { char_trace: utf32_trace, character_bytes }
        },
    };

    rsx! {
        div { class: "char-steps",
            h2 { "{character_label}" }
            {encoding_steps}
        }
    }
}
