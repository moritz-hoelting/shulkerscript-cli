use crate::subcommands::{self, BuildArgs, CleanArgs, InitArgs};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Command,
    /// Enable tracing output
    ///
    /// When specified without a value, defaults to `info`.
    #[arg(
        long,
        global = true,
        default_missing_value = "info",
        require_equals = true,
        num_args = 0..=1,
        value_name = "LEVEL"
    )]
    trace: Option<TracingLevel>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Initialize a new project.
    Init(InitArgs),
    /// Build the project.
    Build(BuildArgs),
    /// Clean build artifacts.
    /// This will remove the `dist` directory.
    Clean(CleanArgs),
    #[cfg(feature = "watch")]
    /// Watch for changes and execute commands.
    Watch(subcommands::WatchArgs),
    #[cfg(feature = "lang-debug")]
    /// Build the project and dump the intermediate state.
    LangDebug(subcommands::LangDebugArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum TracingLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl Args {
    pub fn run(&self) -> Result<()> {
        if let Some(level) = self.trace {
            setup_tracing(level)?;
        }

        self.cmd.run()
    }
}

impl Command {
    pub fn run(&self) -> Result<()> {
        match self {
            Command::Init(args) => subcommands::init(args)?,
            Command::Build(args) => subcommands::build(args)?,
            Command::Clean(args) => subcommands::clean(args)?,
            #[cfg(feature = "watch")]
            Command::Watch(args) => subcommands::watch(args)?,
            #[cfg(feature = "lang-debug")]
            Command::LangDebug(args) => subcommands::lang_debug(args)?,
        }

        Ok(())
    }
}

impl From<TracingLevel> for Level {
    fn from(value: TracingLevel) -> Self {
        match value {
            TracingLevel::Trace => Level::TRACE,
            TracingLevel::Debug => Level::DEBUG,
            TracingLevel::Info => Level::INFO,
            TracingLevel::Warn => Level::WARN,
            TracingLevel::Error => Level::ERROR,
        }
    }
}

fn setup_tracing(level: TracingLevel) -> Result<()> {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::from(level))
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Args::command().debug_assert();
    }
}
