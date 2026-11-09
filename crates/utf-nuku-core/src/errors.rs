//! DecodeError: what went wrong while decoding.

use std::fmt;

/// Everything that can go wrong while decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    // ---------- any encoding ----------
    /// The input ends in the middle of a char.
    UnexpectedEnd,
    /// The number is not a valid char (above 0x10FFFF, or a UTF-16 surrogate).
    InvalidCodePoint,

    // ---------- UTF-8 ----------
    /// A char can't start with this byte: 10xxxxxx (continuation) or 11111xxx.
    InvalidStartByte,
    /// Inside a char we expected 10xxxxxx but got something else.
    ExpectedContinuation,
    /// The char used more bytes than needed (e.g. '/' as 2 bytes). Forbidden.
    Overlong,

    // ---------- UTF-16 ----------
    /// A low surrogate (110111xx_xxxxxxxx) with no high surrogate before it.
    UnpairedLowSurrogate,
    /// After a high surrogate we expected a low surrogate but got something else.
    ExpectedLowSurrogate,
}

// Display = the human-readable message, used by the CLI (`println!("{error}")`).
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            DecodeError::UnexpectedEnd => "input ends in the middle of a char",
            DecodeError::InvalidCodePoint => "not a valid char",
            DecodeError::InvalidStartByte => "a char can't start with this byte",
            DecodeError::ExpectedContinuation => "expected a continuation byte 10xxxxxx",
            DecodeError::Overlong => "char uses more bytes than needed",
            DecodeError::UnpairedLowSurrogate => "low surrogate without a high surrogate before it",
            DecodeError::ExpectedLowSurrogate => "expected a low surrogate 110111xx_xxxxxxxx",
        };
        write!(f, "{message}")
    }
}

// Lets DecodeError work with `?` and other error tools.
impl std::error::Error for DecodeError {}
