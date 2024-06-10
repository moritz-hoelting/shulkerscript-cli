use std::{env, path::PathBuf};

use color_eyre::eyre::Result;
use path_absolutize::Absolutize;

use crate::{error::Error, terminal_output::print_info};

use super::BuildArgs;

#[derive(Debug, clap::Args, Clone)]
pub struct PackageArgs {
    #[clap(flatten)]
    build_args: BuildArgs,
}

pub fn package(_verbose: bool, args: &PackageArgs) -> Result<()> {
    let path = args.build_args.path.as_path();
    let dist_path = args
        .build_args
        .output
        .clone()
        .or_else(|| env::var("DATAPACK_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| path.join("dist"));

    print_info(&format!(
        "Packaging project at {}",
        path.absolutize()?.display()
    ));

    let (project_config, toml_path) = super::build::get_pack_config(path)?;

    let script_paths = super::build::get_script_paths(
        &toml_path
            .parent()
            .ok_or(Error::InvalidPackPathError(path.to_path_buf()))?
            .join("src"),
    )?;

    let compiled = shulkerscript_lang::compile(&script_paths)?;

    let dist_path = dist_path.join(project_config.pack.name + ".zip");

    compiled.zip(&dist_path)?;

    print_info(&format!(
        "Finished packaging project to {}",
        dist_path.absolutize_from(path)?.display()
    ));

    Ok(())
}
