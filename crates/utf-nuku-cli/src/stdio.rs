//! Bytes in and out. The only file that touches stdin/stdout/stderr.

use std::io::{Read, Write};

/// Read all of stdin. Waits until EOF (end of pipe/file, or Ctrl-D).
pub fn read_stdin() -> Result<Vec<u8>, std::io::Error> {
    let mut input_bytes = Vec::new();
    std::io::stdin().read_to_end(&mut input_bytes)?;
    Ok(input_bytes)
}

/// Write bytes to stdout exactly as they are, nothing added.
pub fn write_stdout(output_bytes: &[u8]) -> Result<(), std::io::Error> {
    let mut stdout = std::io::stdout();
    stdout.write_all(output_bytes)?;
    stdout.flush()?;
    Ok(())
}

/// Write text to stderr: for -v steps and error messages, never mixed into stdout's bytes.
pub fn write_stderr(text: &str) -> Result<(), std::io::Error> {
    let mut stderr = std::io::stderr();
    stderr.write_all(text.as_bytes())?;
    stderr.flush()?;
    Ok(())
}
