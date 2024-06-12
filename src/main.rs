mod cli;
mod config;
mod error;
mod subcommands;
mod terminal_output;

use std::process::ExitCode;

use clap::Parser;
use cli::Args;

fn main() -> ExitCode {
    color_eyre::install().unwrap();
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    match args.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
