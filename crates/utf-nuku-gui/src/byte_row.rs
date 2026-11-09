//! All bytes, grouped per char. Click a group → it becomes the selected char.
//!
//! ```text
//!   'H'      'é'          '😀'
//!   48    │  c3 a9     │  f0 9f 98 80
//! ```

use dioxus::prelude::*;
use utf_nuku_core::ByteOrder;

use crate::char_trace::CharTrace;

#[component]
pub fn ByteRow(
    char_traces: Vec<CharTrace>,
    byte_order: ByteOrder,
    selected_char_index: Signal<Option<usize>>,
) -> Element {
    rsx! {
        div { class: "byte-row",
            for (char_index, char_trace) in char_traces.into_iter().enumerate() {
                CharGroup { char_index, char_trace, byte_order, selected_char_index }
            }
        }
    }
}

/// One char: its label on top, its bytes below (hex + bits).
#[component]
fn CharGroup(
    char_index: usize,
    char_trace: CharTrace,
    byte_order: ByteOrder,
    mut selected_char_index: Signal<Option<usize>>,
) -> Element {
    let is_selected = selected_char_index() == Some(char_index);
    let group_class = if is_selected {
        "char-group selected"
    } else {
        "char-group"
    };
    let character_label = format!("{:?}", char_trace.character());
    let character_bytes = char_trace.character_bytes(byte_order);

    rsx! {
        div {
            class: "{group_class}",
            onclick: move |_| selected_char_index.set(Some(char_index)),
            div { class: "char-label", "{character_label}" }
            div { class: "char-bytes",
                for byte in character_bytes {
                    div { class: "byte",
                        div { class: "byte-hex", "{byte:02x}" }
                        div { class: "byte-bits", "{byte:08b}" }
                    }
                }
            }
        }
    }
}
