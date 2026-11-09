//! Input box, Encode / Decode mode switch, encoding + byte order choices.
//! No run buttons: every change here updates the result right away (see app.rs).

use dioxus::prelude::*;
use utf_nuku_core::{ByteOrder, Encoding};

use crate::view_options::Mode;

/// select value → Encoding
fn encoding_from_value(select_value: &str) -> Encoding {
    match select_value {
        "utf16" => Encoding::Utf16,
        "utf32" => Encoding::Utf32,
        _ => Encoding::Utf8,
    }
}

/// select value → ByteOrder
fn byte_order_from_value(select_value: &str) -> ByteOrder {
    match select_value {
        "le" => ByteOrder::Little,
        _ => ByteOrder::Big,
    }
}

#[component]
pub fn Controls(
    mut input_text: Signal<String>,
    mut mode: Signal<Mode>,
    mut encoding: Signal<Encoding>,
    mut byte_order: Signal<ByteOrder>,
) -> Element {
    let encode_button_class = if mode() == Mode::Encode { "active" } else { "" };
    let decode_button_class = if mode() == Mode::Decode { "active" } else { "" };
    let placeholder_text = match mode() {
        Mode::Encode => "text, e.g. Hé😀",
        Mode::Decode => "bytes as hex (48 c3 a9) or bits (01001000 11000011)",
    };

    rsx! {
        div { class: "controls",
            div { class: "options",
                button {
                    class: "{encode_button_class}",
                    onclick: move |_| mode.set(Mode::Encode),
                    "Encode"
                }
                button {
                    class: "{decode_button_class}",
                    onclick: move |_| mode.set(Mode::Decode),
                    "Decode"
                }
                select {
                    onchange: move |event| encoding.set(encoding_from_value(&event.value())),
                    option { value: "utf8", "UTF-8" }
                    option { value: "utf16", "UTF-16" }
                    option { value: "utf32", "UTF-32" }
                }
                select {
                    onchange: move |event| byte_order.set(byte_order_from_value(&event.value())),
                    option { value: "be", "Big endian" }
                    option { value: "le", "Little endian" }
                }
            }
            input {
                class: "text-input",
                value: "{input_text}",
                placeholder: "{placeholder_text}",
                oninput: move |event| input_text.set(event.value()),
            }
        }
    }
}
