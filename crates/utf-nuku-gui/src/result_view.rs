//! Everything below the controls: error, text, all bytes, per-char bytes, selected char's steps.

use dioxus::prelude::*;

use crate::all_bytes_view::AllBytesView;
use crate::byte_row::ByteRow;
use crate::char_steps::CharSteps;
use crate::core_calls::RunResult;
use crate::view_options::ByteFormat;

#[component]
pub fn ResultView(
    run_result: RunResult,
    byte_format: Signal<ByteFormat>,
    selected_char_index: Signal<Option<usize>>,
) -> Element {
    // the text these traces stand for
    let mut result_text = String::new();
    for char_trace in &run_result.char_traces {
        result_text.push(char_trace.character());
    }

    // the selected char's trace, if the index is still valid
    let mut selected_char_trace = None;
    if let Some(char_index) = selected_char_index() {
        selected_char_trace = run_result.char_traces.get(char_index).cloned();
    }

    let byte_order = run_result.byte_order;
    let has_chars = !run_result.char_traces.is_empty();

    rsx! {
        div { class: "result",
            if let Some(error_message) = run_result.error_message.clone() {
                div { class: "error", "⚠ {error_message}" }
            }
            div { class: "result-text",
                span { class: "section-title", "text " }
                "{result_text}"
            }
            AllBytesView { run_result: run_result.clone(), byte_format, selected_char_index }
            if has_chars {
                div { class: "section-title", "per char (click one)" }
                ByteRow { char_traces: run_result.char_traces.clone(), byte_order, selected_char_index }
            }
            if let Some(char_trace) = selected_char_trace {
                CharSteps { char_trace, byte_order }
            } else if has_chars {
                p { class: "hint", "Click a char above to see its steps." }
            }
        }
    }
}
