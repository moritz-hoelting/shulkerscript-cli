use std::path::PathBuf;

use crate::{error::Result, subcommands};
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
    /// Initialize a new project in the current directory.
    Init {
        /// The path of the folder to initialize in.
        #[clap(default_value = ".")]
        path: PathBuf,
        /// The name of the project.
        #[clap(short, long)]
        name: Option<String>,
        /// The description of the project.
        #[clap(short, long)]
        description: Option<String>,
        /// The pack format version.
        #[clap(short, long)]
        pack_format: Option<u8>,
        /// Force initialization even if the directory is not empty.
        #[clap(short, long)]
        force: bool,
    },
}

impl Args {
    pub fn run(&self) -> Result<()> {
        match &self.cmd {
            Command::Init {
                path,
                name,
                description,
                pack_format,
                force,
            } => subcommands::init(
                self.verbose,
                path,
                name.as_deref(),
                description.as_deref(),
                *pack_format,
                *force,
            )?,
        }

        Ok(())
    }
}
