use crate::{
    error::Result,
    subcommands::{self, CompileArgs, InitArgs},
};
use clap::{Parser, Subcommand};

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
    /// Compile the project.
    Compile(CompileArgs),
    #[cfg(feature = "zip")]
    /// Compile and package the project.
    Package(subcommands::PackageArgs),
    #[cfg(feature = "lang-debug")]
    /// Compile the project and dump the intermediate state.
    LangDebug(subcommands::LangDebugArgs),
}

impl Args {
    pub fn run(&self) -> Result<()> {
        match &self.cmd {
            Command::Init(args) => subcommands::init(self.verbose, args)?,
            Command::Compile(args) => subcommands::compile(self.verbose, args)?,
            #[cfg(feature = "zip")]
            Command::Package(args) => subcommands::package(self.verbose, args)?,
            #[cfg(feature = "lang-debug")]
            Command::LangDebug(args) => subcommands::lang_debug(args)?,
        }

        Ok(())
    }
}
