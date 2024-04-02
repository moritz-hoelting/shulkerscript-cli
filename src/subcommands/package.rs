use crate::error::Result;
use std::path::PathBuf;

#[derive(Debug, clap::Args, Clone)]
pub struct PackageArgs {
    /// The path of the project to package.
    #[clap(default_value = ".")]
    path: PathBuf,
}

pub fn package(_verbose: bool, _args: &PackageArgs) -> Result<()> {
    println!("PACKAGE");

    Ok(())
}
