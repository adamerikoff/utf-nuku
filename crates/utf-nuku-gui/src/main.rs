//! utf-nuku-gui: type text or bytes, pick Encode / Decode, see the result live, click a char to see its steps.

mod all_bytes_view;
mod app;
mod byte_input;
mod byte_row;
mod char_steps;
mod char_trace;
mod controls;
mod core_calls;
mod result_view;
mod step_parts;
mod utf16_steps;
mod utf32_steps;
mod utf8_steps;
mod view_options;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

fn main() {
    // big enough to show the default text's whole result without resizing
    let window = WindowBuilder::new()
        .with_title("utf-nuku")
        .with_inner_size(LogicalSize::new(1000.0, 860.0));
    let config = Config::new().with_window(window);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::App);
}
