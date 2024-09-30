//! This crate is the cli app of the Shulkerscript language for creating Minecraft datapacks.
//!
//! # Installation
//! ```bash
//! cargo install shulkerscript-cli
//! ```
//!
//! # Usage
//! An extended description of the commands can be found in the readme or by running `shulkerscript --help`.
//!
//! ### Initialize a new project
//! ```bash
//! shulkerscript init [OPTIONS] [PATH]
//! ```
//!
//! ### Build a project
//! ```bash
//! shulkerscript build [OPTIONS] [PATH]
//! ```
//!
//! ### Clean the output directory
//! ```bash
//! shulkerscript clean [OPTIONS] [PATH]
//! ```
//!
//! ### Watch for changes
//! ```bash
//! shulkerscript watch [OPTIONS] [PATH]
//! ```

pub mod cli;
pub mod config;
pub mod error;
pub mod subcommands;
pub mod terminal_output;
pub mod util;
