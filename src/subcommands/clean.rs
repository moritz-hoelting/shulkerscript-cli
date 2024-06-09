use std::path::PathBuf;

use color_eyre::eyre::Result;
use path_absolutize::Absolutize;

use crate::{
    error::Error,
    terminal_output::{print_error, print_info},
};

#[derive(Debug, clap::Args, Clone)]
pub struct CleanArgs {
    /// The path of the project to clean.
    #[clap(default_value = ".")]
    pub path: PathBuf,
}

pub fn clean(_verbose: bool, args: &CleanArgs) -> Result<()> {
    let path = args.path.as_path();

    print_info(&format!(
        "Cleaning project at {}",
        path.absolutize_from(path)?.display()
    ));

    let dist_path = path.join("dist");

    if !path.join("pack.toml").exists() {
        print_error("The specified directory is not a ShulkerScript project.");
        return Err(Error::InvalidPackPathError(path.to_path_buf()).into());
    }

    if dist_path.exists() {
        std::fs::remove_dir_all(&dist_path)?;
    }

    print_info("Project cleaned successfully.");

    Ok(())
}
