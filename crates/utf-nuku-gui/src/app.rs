//! The whole window: holds the state (signals) and lays out the parts.

use dioxus::prelude::*;
use utf_nuku_core::{ByteOrder, Encoding};

use crate::controls::Controls;
use crate::core_calls::{run_decode, run_encode};
use crate::result_view::ResultView;
use crate::view_options::{ByteFormat, Mode};

/// The CSS file, baked into the binary at compile time.
const STYLE_CSS: &str = include_str!("../assets/style.css");

/// The page's CSS. Its own component with no props, so it's drawn once
/// and never re-sent when the rest of the page redraws.
#[component]
fn GlobalStyle() -> Element {
    rsx! {
        document::Style { "{STYLE_CSS}" }
    }
}

#[component]
pub fn App() -> Element {
    // state: when one of these changes, everything that reads it is drawn again
    let input_text = use_signal(|| String::from("Hé😀"));
    let mode = use_signal(|| Mode::Encode);
    let encoding = use_signal(|| Encoding::Utf8);
    let byte_order = use_signal(|| ByteOrder::Big);
    let byte_format = use_signal(|| ByteFormat::Hex);
    let selected_char_index = use_signal(|| Some(0));

    // the result is computed from the state: use_memo re-runs this
    // every time one of the signals read inside changes → live updates
    let run_result = use_memo(move || match mode() {
        Mode::Encode => run_encode(&input_text(), encoding(), byte_order()),
        Mode::Decode => run_decode(&input_text(), encoding(), byte_order()),
    });

    rsx! {
        GlobalStyle {}
        div { class: "app",
            h1 { "utf-nuku" }
            Controls { input_text, mode, encoding, byte_order }
            ResultView { run_result: run_result(), byte_format, selected_char_index }
        }
    }
}
