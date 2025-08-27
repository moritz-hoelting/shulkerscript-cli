use std::process::ExitCode;

use clap::Parser;

use shulkerscript_cli::{cli::Cli, terminal_output::print_info};

fn main() -> ExitCode {
    human_panic::setup_panic!(human_panic::Metadata::new(
        env!("CARGO_PKG_NAME"),
        const_format::formatcp!(
            "{cli_version};lib={lib_version}",
            cli_version = env!("CARGO_PKG_VERSION"),
            lib_version = shulkerscript::VERSION
        ),
    )
    .authors(env!("CARGO_PKG_AUTHORS").replace(":", ", "))
    .homepage(env!("CARGO_PKG_HOMEPAGE")));

    if dotenvy::dotenv().is_ok() {
        print_info("Using environment variables from .env file");
    }

    let args = Cli::parse();

    match args.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
