//! Every byte of the result in one box: hex or bits, selectable, with a Copy button.
//! Copied hex/bits can be pasted straight back into Decode mode.
//! Each char's bytes are their own span: the selected char's bytes get a box,
//! and clicking bytes here selects that char.

use dioxus::prelude::*;

use crate::core_calls::RunResult;
use crate::view_options::ByteFormat;

/// Bytes as one line of text.
///
/// ```text
/// Hex  → "48 c3 a9"
/// Bits → "01001000 11000011 10101001"
/// ```
fn bytes_text(bytes: &[u8], byte_format: ByteFormat) -> String {
    let mut byte_texts = Vec::new();
    for byte in bytes {
        let byte_text = match byte_format {
            ByteFormat::Hex => format!("{byte:02x}"),
            ByteFormat::Bits => format!("{byte:08b}"),
        };
        byte_texts.push(byte_text);
    }
    byte_texts.join(" ")
}

#[component]
pub fn AllBytesView(
    run_result: RunResult,
    mut byte_format: Signal<ByteFormat>,
    mut selected_char_index: Signal<Option<usize>>,
) -> Element {
    // one text per char: that char's bytes in the chosen format
    let mut char_bytes_texts = Vec::new();
    let mut byte_count = 0;
    for char_trace in &run_result.char_traces {
        let character_bytes = char_trace.character_bytes(run_result.byte_order);
        byte_count += character_bytes.len();
        char_bytes_texts.push(bytes_text(&character_bytes, byte_format()));
    }

    // the text to copy: all chars' bytes, space-separated
    let copy_text = char_bytes_texts.join(" ");

    let hex_button_class = if byte_format() == ByteFormat::Hex {
        "active"
    } else {
        ""
    };
    let bits_button_class = if byte_format() == ByteFormat::Bits {
        "active"
    } else {
        ""
    };

    rsx! {
        div { class: "all-bytes",
            div { class: "all-bytes-header",
                span { class: "section-title", "all bytes ({byte_count})" }
                button {
                    class: "{hex_button_class}",
                    onclick: move |_| byte_format.set(ByteFormat::Hex),
                    "Hex"
                }
                button {
                    class: "{bits_button_class}",
                    onclick: move |_| byte_format.set(ByteFormat::Bits),
                    "Bits"
                }
                button {
                    onclick: move |_| {
                        // ask the webview to put the text on the clipboard
                        // ({:?} writes it as a quoted, escaped string, which JavaScript accepts)
                        let copy_script = format!("navigator.clipboard.writeText({copy_text:?})");
                        let _ = document::eval(&copy_script);
                    },
                    "Copy"
                }
            }
            div { class: "all-bytes-text",
                for (char_index, char_bytes_text) in char_bytes_texts.into_iter().enumerate() {
                    span {
                        class: if selected_char_index() == Some(char_index) { "char-bytes-text selected" } else { "char-bytes-text" },
                        onclick: move |_| selected_char_index.set(Some(char_index)),
                        "{char_bytes_text}"
                    }
                    " "
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_and_bits_text() {
        let all_bytes = [0b0100_1000, 0b1100_0011, 0b1010_1001];
        assert_eq!(bytes_text(&all_bytes, ByteFormat::Hex), "48 c3 a9");
        assert_eq!(
            bytes_text(&all_bytes, ByteFormat::Bits),
            "01001000 11000011 10101001"
        );
    }
}
