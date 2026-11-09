//! Small building blocks for the step views: a labeled row, and colored bits.

use dioxus::prelude::*;

/// `value` as exactly `bit_count` bits, no underscores (so header + data read as one byte).
///
/// ```text
/// plain_bits(0b11, 5) → "00011"
/// ```
pub fn plain_bits(value: u32, bit_count: u32) -> String {
    let width = bit_count as usize;
    format!("{value:0width$b}")
}

/// One step: label on the left, value on the right.
#[component]
pub fn StepRow(#[props(into)] label: String, children: Element) -> Element {
    rsx! {
        div { class: "step",
            div { class: "step-label", "{label}" }
            div { class: "step-value", {children} }
        }
    }
}

/// A byte or unit drawn as header (one color) + data (another color).
///
/// ```text
/// header "110", data "00011"  →  [110][00011]
/// ```
#[component]
pub fn HeaderDataBits(
    #[props(into)] header_text: String,
    #[props(into)] data_text: String,
) -> Element {
    rsx! {
        span { class: "bits",
            span { class: "header-bits", "{header_text}" }
            span { class: "data-bits", "{data_text}" }
        }
    }
}

/// Data bits only (code point, split pieces).
#[component]
pub fn DataBits(#[props(into)] data_text: String) -> Element {
    rsx! {
        span { class: "bits data-bits", "{data_text}" }
    }
}

/// The char's bytes in hex: "c3 a9".
pub fn hex_text(character_bytes: &[u8]) -> String {
    let mut byte_texts = Vec::new();
    for byte in character_bytes {
        byte_texts.push(format!("{byte:02x}"));
    }
    byte_texts.join(" ")
}
