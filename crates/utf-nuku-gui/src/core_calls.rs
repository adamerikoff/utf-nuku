//! The GUI's only contact with core: run encode or decode, get traces back.

use utf_nuku_core::{
    ByteOrder, Encoding, bytes_to_utf16_units, bytes_to_utf32_units, trace_decode_utf8,
    trace_decode_utf16, trace_decode_utf32, trace_encode_utf8, trace_encode_utf16,
    trace_encode_utf32,
};

use crate::byte_input::parse_byte_input;
use crate::char_trace::CharTrace;

/// What the Encode / Decode buttons produce.
#[derive(Debug, Clone, PartialEq)]
pub struct RunResult {
    /// byte order used for this run (UTF-16/32 bytes are shown in it)
    pub byte_order: ByteOrder,
    /// one trace per char (for decode: the chars before an error)
    pub char_traces: Vec<CharTrace>,
    /// set if decode stopped early, or the input couldn't be read
    pub error_message: Option<String>,
}

/// Text → one trace per char. Encoding can't fail.
pub fn run_encode(input_text: &str, encoding: Encoding, byte_order: ByteOrder) -> RunResult {
    let mut char_traces = Vec::new();

    match encoding {
        Encoding::Utf8 => {
            for utf8_trace in trace_encode_utf8(input_text) {
                char_traces.push(CharTrace::Utf8(utf8_trace));
            }
        }
        Encoding::Utf16 => {
            for utf16_trace in trace_encode_utf16(input_text) {
                char_traces.push(CharTrace::Utf16(utf16_trace));
            }
        }
        Encoding::Utf32 => {
            for utf32_trace in trace_encode_utf32(input_text) {
                char_traces.push(CharTrace::Utf32(utf32_trace));
            }
        }
    }

    RunResult {
        byte_order,
        char_traces,
        error_message: None,
    }
}

/// Typed bytes (hex or bits) → one trace per char, until the first error.
pub fn run_decode(input_text: &str, encoding: Encoding, byte_order: ByteOrder) -> RunResult {
    let input_bytes = match parse_byte_input(input_text) {
        Ok(input_bytes) => input_bytes,
        Err(message) => {
            return RunResult {
                byte_order,
                char_traces: Vec::new(),
                error_message: Some(message),
            };
        }
    };

    let mut char_traces = Vec::new();
    let decode_error;

    match encoding {
        Encoding::Utf8 => {
            let (utf8_traces, utf8_error) = trace_decode_utf8(&input_bytes);
            for utf8_trace in utf8_traces {
                char_traces.push(CharTrace::Utf8(utf8_trace));
            }
            decode_error = utf8_error;
        }
        Encoding::Utf16 => match bytes_to_utf16_units(&input_bytes, byte_order) {
            Ok(utf16_units) => {
                let (utf16_traces, utf16_error) = trace_decode_utf16(&utf16_units);
                for utf16_trace in utf16_traces {
                    char_traces.push(CharTrace::Utf16(utf16_trace));
                }
                decode_error = utf16_error;
            }
            Err(error) => decode_error = Some(error),
        },
        Encoding::Utf32 => match bytes_to_utf32_units(&input_bytes, byte_order) {
            Ok(utf32_units) => {
                let (utf32_traces, utf32_error) = trace_decode_utf32(&utf32_units);
                for utf32_trace in utf32_traces {
                    char_traces.push(CharTrace::Utf32(utf32_trace));
                }
                decode_error = utf32_error;
            }
            Err(error) => decode_error = Some(error),
        },
    }

    // DecodeError → its message (from its Display)
    let error_message = match decode_error {
        Some(error) => Some(error.to_string()),
        None => None,
    };

    RunResult {
        byte_order,
        char_traces,
        error_message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode_same_chars() {
        let encode_result = run_encode("Hé😀", Encoding::Utf16, ByteOrder::Big);
        assert_eq!(encode_result.char_traces.len(), 3);

        let decode_result = run_decode("00 48 00 e9 d8 3d de 00", Encoding::Utf16, ByteOrder::Big);
        assert_eq!(decode_result, encode_result);
    }

    #[test]
    fn decode_keeps_chars_before_error() {
        let decode_result = run_decode("48 80", Encoding::Utf8, ByteOrder::Big);
        assert_eq!(decode_result.char_traces.len(), 1);
        assert!(decode_result.error_message.is_some());
    }

    #[test]
    fn decode_bad_input() {
        let decode_result = run_decode("zz", Encoding::Utf8, ByteOrder::Big);
        assert!(decode_result.char_traces.is_empty());
        assert!(decode_result.error_message.is_some());
    }
}
