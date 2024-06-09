use color_eyre::eyre::Result;
use path_absolutize::Absolutize;

use crate::{
    config::ProjectConfig,
    error::Error,
    terminal_output::{print_error, print_info},
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
}

pub fn build(_verbose: bool, args: &BuildArgs) -> Result<()> {
    let path = args.path.as_path();

    print_info(&format!(
        "Building project at {}",
        path.absolutize()?.display()
    ));

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

    env::set_current_dir(
        toml_path
            .parent()
            .expect("Failed to get parent directory of pack.toml"),
    )?;

    let toml_content = fs::read_to_string(&toml_path)?;
    let project_config = toml::from_str::<ProjectConfig>(&toml_content)?;

    let script_paths = get_script_paths(
        &toml_path
            .parent()
            .ok_or(Error::InvalidPackPathError(path.to_path_buf()))?
            .join("src"),
    )?;

    let compiled = shulkerscript_lang::compile(&script_paths)?;

    let dist_path = toml_path
        .parent()
        .expect("Failed to get parent directory of pack.toml")
        .join("dist")
        .join(project_config.pack.name);

    compiled.place(&dist_path)?;

    print_info(&format!(
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
