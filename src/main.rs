use std::process::ExitCode;

use clap::Parser;

use shulkerscript_cli::{cli::Args, terminal_output::print_info};

fn main() -> ExitCode {
    human_panic::setup_panic!();
    if dotenvy::dotenv().is_ok() {
        print_info("Using environment variables from .env file");
    }

    let args = Args::parse();

    match args.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
