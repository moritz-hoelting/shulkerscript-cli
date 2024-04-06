use crate::error::Result;

use super::BuildArgs;

#[derive(Debug, clap::Args, Clone)]
pub struct PackageArgs {
    #[clap(flatten)]
    build_args: BuildArgs,
}

pub fn package(_verbose: bool, args: &PackageArgs) -> Result<()> {
    println!("PACKAGE");
    println!("  - Args: {:?}", args);

    Ok(())
}
