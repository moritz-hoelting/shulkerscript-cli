use color_eyre::eyre::Result;
use path_absolutize::Absolutize;
use shulkerbox::virtual_fs::VFolder;

use crate::{
    config::ProjectConfig,
    error::Error,
    terminal_output::{print_error, print_info, print_warning},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, clap::Args, Clone)]
pub struct BuildArgs {
    /// The path of the project to build.
    #[clap(default_value = ".")]
    pub path: PathBuf,
    /// The path of the directory to place the compiled datapack.
    /// Overrides the `DATAPACK_DIR` environment variable.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    /// The path of a folder which files and subfolders will be copied to the root of the datapack.
    /// Overrides the `assets` field in the pack.toml file.
    #[clap(short, long)]
    pub assets: Option<PathBuf>,
}

pub fn build(_verbose: bool, args: &BuildArgs) -> Result<()> {
    let path = args.path.as_path();
    let dist_path = args
        .output
        .clone()
        .or_else(|| env::var("DATAPACK_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| path.join("dist"));

    print_info(format!(
        "Building project at {}",
        path.absolutize()?.display()
    ));

    // env::set_current_dir(
    //     toml_path
    //         .parent()
    //         .expect("Failed to get parent directory of pack.toml"),
    // )?;

    let (project_config, toml_path) = get_pack_config(path)?;

    let script_paths = get_script_paths(
        &toml_path
            .parent()
            .ok_or(Error::InvalidPackPathError(path.to_path_buf()))?
            .join("src"),
    )?;

    let compiled = shulkerscript_lang::compile(&script_paths)?;

    let assets_path = args.assets.clone().or(project_config
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

    let dist_path = dist_path.join(project_config.pack.name);

    output.place(&dist_path)?;

    print_info(format!(
        "Finished building project to {}",
        dist_path.absolutize_from(path)?.display()
    ));

    Ok(())
}

/// Recursively get all script paths in a directory.
pub(super) fn get_script_paths(path: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    _get_script_paths(path, "")
}

fn _get_script_paths(path: &Path, prefix: &str) -> std::io::Result<Vec<(String, PathBuf)>> {
    if path.exists() && path.is_dir() {
        let contents = path.read_dir()?;

        let mut paths = Vec::new();

        for entry in contents {
            let path = entry?.path();
            if path.is_dir() {
                let prefix = path
                    .absolutize()?
                    .file_name()
                    .unwrap()
                    .to_str()
                    .expect("Invalid folder name")
                    .to_string()
                    + "/";
                paths.extend(_get_script_paths(&path, &prefix)?);
            } else if path.extension().unwrap_or_default() == "shu" {
                paths.push((
                    prefix.to_string()
                        + path
                            .file_stem()
                            .expect("ShulkerScript files are not allowed to have empty names")
                            .to_str()
                            .expect("Invalid characters in filename"),
                    path,
                ));
            }
        }

        Ok(paths)
    } else {
        Ok(Vec::new())
    }
}

/// Get the pack config and config path from a project path.
///
/// # Errors
/// - If the specified path does not exist.
/// - If the specified directory does not contain a pack.toml file.
pub(super) fn get_pack_config(path: &Path) -> Result<(ProjectConfig, PathBuf)> {
    let toml_path = if !path.exists() {
        print_error("The specified path does not exist.");
        return Err(Error::PathNotFoundError(path.to_path_buf()))?;
    } else if path.is_dir() {
        let toml_path = path.join("pack.toml");
        if !toml_path.exists() {
            print_error("The specified directory does not contain a pack.toml file.");
            Err(Error::InvalidPackPathError(path.to_path_buf()))?;
        }
        toml_path
    } else if path.is_file()
        && path
            .file_name()
            .ok_or(Error::InvalidPackPathError(path.to_path_buf()))?
            == "pack.toml"
    {
        path.to_path_buf()
    } else {
        print_error("The specified path is neither a directory nor a pack.toml file.");
        return Err(Error::InvalidPackPathError(path.to_path_buf()))?;
    };

    let toml_content = fs::read_to_string(&toml_path)?;
    let project_config = toml::from_str::<ProjectConfig>(&toml_content)?;

    Ok((project_config, toml_path))
}
