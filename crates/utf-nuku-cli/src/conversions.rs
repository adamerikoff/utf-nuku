//! CLI types → core types. The only file that knows both sides.

use crate::cli_args::{CliByteOrder, CliEncoding};
use utf_nuku_core::{ByteOrder, Encoding};

pub fn to_core_encoding(cli_encoding: CliEncoding) -> Encoding {
    match cli_encoding {
        CliEncoding::Utf8 => Encoding::Utf8,
        CliEncoding::Utf16 => Encoding::Utf16,
        CliEncoding::Utf32 => Encoding::Utf32,
    }
}

pub fn to_core_byte_order(cli_byte_order: CliByteOrder) -> ByteOrder {
    match cli_byte_order {
        CliByteOrder::Le => ByteOrder::Little,
        CliByteOrder::Be => ByteOrder::Big,
    }
}
