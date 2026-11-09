//! The 2 commands: stdin → core → stdout.
//! With -v, every step also goes to stderr (so stdout stays clean for files and pipes).

use std::error::Error;

use utf_nuku_core::{decode_from_bytes, decode_utf8, encode_to_bytes};

use crate::cli_args::EncodingOptions;
use crate::conversions::{to_core_byte_order, to_core_encoding};
use crate::decode_trace_text::decode_trace_text;
use crate::encode_trace_text::encode_trace_text;
use crate::stdio::{read_stdin, write_stderr, write_stdout};

/// -v output: the steps + a newline, on stderr. Nothing for empty input.
fn write_trace_text(trace_text: &str) -> Result<(), std::io::Error> {
    if trace_text.is_empty() {
        return Ok(());
    }
    write_stderr(&format!("{trace_text}\n"))
}

/// stdin bytes (UTF-8 from the terminal) → text → encoded bytes → stdout
pub fn run_encode(encoding_options: &EncodingOptions) -> Result<(), Box<dyn Error>> {
    let input_bytes = read_stdin()?;

    // the terminal sends text as UTF-8 bytes → String, using our own decoder
    let text = decode_utf8(&input_bytes)?;

    let encoding = to_core_encoding(encoding_options.encoding);
    let byte_order = to_core_byte_order(encoding_options.byte_order);
    let encoded_bytes = encode_to_bytes(&text, encoding, byte_order);

    if encoding_options.verbose {
        let trace_text = encode_trace_text(&text, encoding);
        write_trace_text(&trace_text)?;
    }

    write_stdout(&encoded_bytes)?;
    Ok(())
}

/// stdin encoded bytes → text → stdout (as UTF-8, what the terminal shows)
pub fn run_decode(encoding_options: &EncodingOptions) -> Result<(), Box<dyn Error>> {
    let encoded_bytes = read_stdin()?;

    let encoding = to_core_encoding(encoding_options.encoding);
    let byte_order = to_core_byte_order(encoding_options.byte_order);

    // before decoding: on an error, the steps up to it are shown first, then main prints the error
    if encoding_options.verbose {
        let trace_text = decode_trace_text(&encoded_bytes, encoding, byte_order)?;
        write_trace_text(&trace_text)?;
    }

    let text = decode_from_bytes(&encoded_bytes, encoding, byte_order)?;

    // a String is UTF-8 inside: as_bytes() gives those bytes
    write_stdout(text.as_bytes())?;
    Ok(())
}
