//! utf_nuku_core: UTF-8/16/32 encoder and decoder with step traces.
//!
//! All modules are private. The public API is the `pub use` list below,
//! so callers write `utf_nuku_core::encode_utf8(…)`.

// shared by every encoding
mod bit_text;
mod bits;
mod byte_order;
mod errors;
mod unicode;

// UTF-8
mod utf8_common;
mod utf8_decode;
mod utf8_encode;
mod utf8_trace;

// UTF-16
mod utf16_common;
mod utf16_decode;
mod utf16_encode;
mod utf16_trace;

// UTF-32
mod utf32;
mod utf32_trace;

// text ↔ bytes for any encoding
mod encoding;

// ---------- public API ----------
pub use byte_order::ByteOrder;
pub use byte_order::{
    bytes_to_utf16_units, bytes_to_utf32_units, utf16_units_to_bytes, utf32_units_to_bytes,
};
pub use encoding::{Encoding, decode_from_bytes, encode_to_bytes};
pub use errors::DecodeError;
pub use utf8_decode::decode_utf8;
pub use utf8_encode::encode_utf8;
pub use utf16_decode::decode_utf16;
pub use utf16_encode::encode_utf16;
pub use utf32::{decode_utf32, encode_utf32};

// numbers → bit text (for showing traces)
pub use bit_text::{bits_text, utf8_code_point_bit_count};

// step-by-step traces: one per char, same struct for encode and decode (for CLI -v and the GUI)
pub use utf8_decode::trace_decode_utf8;
pub use utf8_encode::trace_encode_utf8;
pub use utf8_trace::Utf8CharTrace;
pub use utf16_decode::trace_decode_utf16;
pub use utf16_encode::trace_encode_utf16;
pub use utf16_trace::{SurrogatePairTrace, Utf16CharTrace};
pub use utf32::{trace_decode_utf32, trace_encode_utf32};
pub use utf32_trace::Utf32CharTrace;
