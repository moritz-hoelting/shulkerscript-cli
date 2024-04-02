use crate::{
    config::ProjectConfig,
    error::{Error, Result},
    terminal_output::{print_error, print_info},
};
use std::{fs, path::PathBuf};

use crate::util;

#[derive(Debug, clap::Args, Clone)]
pub struct CompileArgs {
    /// The path of the project to compile.
    #[clap(default_value = ".")]
    path: PathBuf,
}

pub fn compile(_verbose: bool, args: &CompileArgs) -> Result<()> {
    let path = args.path.as_path();

    let str_path = util::to_absolute_path(path)?;
    print_info(&format!("Compiling project at {}", str_path));

    let toml_path = if !path.exists() {
        print_error("The specified path does not exist.");
        return Err(Error::PathNotFoundError(path.to_path_buf()));
    } else if path.is_dir() {
        let toml_path = path.join("pack.toml");
        if !toml_path.exists() {
            print_error("The specified directory does not contain a pack.toml file.");
            return Err(Error::InvalidPackPathError(path.to_path_buf()));
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
        return Err(Error::InvalidPackPathError(path.to_path_buf()));
    };

    let toml_content = fs::read_to_string(&toml_path)?;
    let project_config = toml::from_str::<ProjectConfig>(&toml_content)?;

    let main_path = toml_path
        .parent()
        .ok_or(Error::InvalidPackPathError(path.to_path_buf()))?
        .join("src/main.shu");
    let compiled = shulkerscript_lang::compile(&main_path)?;

    let dist_path = toml_path
        .parent()
        .expect("Failed to get parent directory of pack.toml")
        .join("dist")
        .join(project_config.pack.name);

    compiled.place(&dist_path)?;

    print_info(&format!(
        "Finished compiling project to {}",
        util::to_absolute_path(&dist_path)?
    ));

    Ok(())
}
