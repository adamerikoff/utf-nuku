mod cli_args;
mod command;
mod conversions;
mod decode_trace_text;
mod encode_trace_text;
mod stdio;
mod trace_layout;

use std::process::ExitCode;

use clap::Parser;
use cli_args::{Cli, Command};
use command::{run_decode, run_encode};

/// Exit code: 0 = success, 1 = error (clap uses 2 for bad arguments)
fn main() -> ExitCode {
    let cli = Cli::parse();

    let run_result = match &cli.command {
        Command::Encode(encoding_options) => run_encode(encoding_options),
        Command::Decode(encoding_options) => run_decode(encoding_options),
    };

    match run_result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // stderr: the message never ends up inside the output bytes
            eprintln!("utf-nuku: {error}");
            ExitCode::FAILURE
        }
    }
}
