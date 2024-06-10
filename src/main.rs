use std::process::ExitCode;

use clap::Parser;
use shulkerscript::cli::Args;

fn main() -> ExitCode {
    color_eyre::install().unwrap();
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    match args.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
