//! What the user can type. Only this file knows about clap.

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Encode and decode UTF-8/16/32 through pipes
#[derive(Parser, Debug)]
#[command(name = "utf-nuku")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Text from stdin → encoded bytes to stdout
    Encode(EncodingOptions),
    /// Encoded bytes from stdin → text to stdout
    Decode(EncodingOptions),
}

/// Options shared by encode and decode
#[derive(Args, Debug)]
pub struct EncodingOptions {
    /// Which UTF to use
    #[arg(short, long, value_enum, default_value_t = CliEncoding::Utf8)]
    pub encoding: CliEncoding,

    /// Byte order for utf16/utf32 (ignored for utf8)
    #[arg(short, long, value_enum, default_value_t = CliByteOrder::Be)]
    pub byte_order: CliByteOrder,

    /// Show every step for each char (on stderr, so the output stays clean)
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliEncoding {
    /// 1–4 bytes per char
    Utf8,
    /// 2 or 4 bytes per char
    Utf16,
    /// 4 bytes per char
    Utf32,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliByteOrder {
    /// Little endian: right byte first
    Le,
    /// Big endian: left byte first
    Be,
}
