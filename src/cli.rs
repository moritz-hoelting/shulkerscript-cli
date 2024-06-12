use crate::subcommands::{self, BuildArgs, CleanArgs, InitArgs};
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Command,
    /// Enable verbose output.
    #[clap(short, long)]
    verbose: bool,
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
    /// Watch for changes and execute command.
    Watch(subcommands::WatchArgs),
    #[cfg(feature = "lang-debug")]
    /// Build the project and dump the intermediate state.
    LangDebug(subcommands::LangDebugArgs),
}

impl Args {
    pub fn run(&self) -> Result<()> {
        self.cmd.run(self.verbose)
    }
}

impl Command {
    pub fn run(&self, verbose: bool) -> Result<()> {
        match self {
            Command::Init(args) => subcommands::init(verbose, args)?,
            Command::Build(args) => subcommands::build(verbose, args)?,
            Command::Clean(args) => subcommands::clean(verbose, args)?,
            #[cfg(feature = "watch")]
            Command::Watch(args) => subcommands::watch(verbose, args)?,
            #[cfg(feature = "lang-debug")]
            Command::LangDebug(args) => subcommands::lang_debug(args)?,
        }

        Ok(())
    }
}
