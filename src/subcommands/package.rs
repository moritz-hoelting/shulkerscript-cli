use std::{env, path::PathBuf};

use color_eyre::eyre::Result;
use path_absolutize::Absolutize;
use shulkerbox::virtual_fs::VFolder;

use crate::{
    error::Error,
    terminal_output::{print_error, print_info, print_warning},
};

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

    print_info(format!(
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

    let assets_path = args.build_args.assets.clone().or(project_config
        .compiler
        .as_ref()
        .and_then(|c| c.assets.as_ref().map(|p| path.join(p))));

    let output = if let Some(assets_path) = assets_path {
        let assets = VFolder::try_from(assets_path.as_path());
        if assets.is_err() {
            print_error(format!(
                "The specified assets path does not exist: {}",
                assets_path.display()
            ));
        }
        let mut assets = assets?;
        let replaced = assets.merge(compiled);

        for replaced in replaced {
            print_warning(format!(
                "Template file {} was replaced by a file in the compiled datapack",
                replaced
            ));
        }

        assets
    } else {
        compiled
    };

    let dist_path = dist_path.join(project_config.pack.name + ".zip");

    output.zip(&dist_path)?;

    print_info(format!(
        "Finished packaging project to {}",
        dist_path.absolutize_from(path)?.display()
    ));

    Ok(())
}
