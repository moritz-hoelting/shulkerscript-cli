use anyhow::Result;
use path_absolutize::Absolutize;
use shulkerbox::{
    util::compile::CompileOptions,
    virtual_fs::{VFile, VFolder},
};
use shulkerscript::base::{FsProvider, PrintHandler};

use crate::{
    config::ProjectConfig,
    error::Error,
    terminal_output::{print_error, print_info, print_success, print_warning},
    util,
};
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, clap::Args, Clone)]
pub struct BuildArgs {
    /// The path of the project to build.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Path of output directory
    ///
    /// The path of the directory to place the compiled datapack.
    #[arg(short, long, env = "DATAPACK_DIR")]
    pub output: Option<PathBuf>,
    /// Path of the assets folder
    ///
    /// The path of a folder which files and subfolders will be copied to the root of the datapack.
    /// Overrides the `assets` field in the pack.toml file.
    #[arg(short, long)]
    pub assets: Option<PathBuf>,
    /// Package the project to a zip file.
    #[arg(short, long)]
    pub zip: bool,
    /// Skip validating the project for pack format compatibility.
    #[arg(long)]
    pub no_validate: bool,
    /// Check if the project can be built without actually building it.
    #[arg(long)]
    pub check: bool,
}

pub fn build(args: &BuildArgs) -> Result<()> {
    if args.zip && !cfg!(feature = "zip") {
        print_error("The zip feature is not enabled. Please install with the `zip` feature enabled to use the `--zip` option.");
        return Err(Error::FeatureNotEnabledError("zip".to_string()).into());
    }

    let path = util::get_project_path(&args.path).unwrap_or(args.path.clone());
    let dist_path = args
        .output
        .as_ref()
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Owned(path.join("dist")));

    let and_package_msg = if args.zip { " and packaging" } else { "" };

    let mut path_display = format!("{}", path.display());
    if path_display.is_empty() {
        path_display.push('.');
    }

    print_info(format!(
        "Building{and_package_msg} project at {path_display}"
    ));

    let (project_config, toml_path) = get_pack_config(&path)?;

    let script_paths = get_script_paths(
        &toml_path
            .parent()
            .ok_or(Error::InvalidPackPathError(path.to_path_buf()))?
            .join("src"),
    )?;

    let datapack = shulkerscript::transpile(
        &PrintHandler::new(),
        &FsProvider::default(),
        project_config.pack.pack_format,
        &script_paths,
    )?;

    if !args.no_validate && !datapack.validate() {
        print_warning(format!(
            "The datapack is not compatible with the specified pack format: {}",
            project_config.pack.pack_format
        ));
        return Err(Error::IncompatiblePackVersionError.into());
    }

    let mut compiled = datapack.compile(&CompileOptions::default());

    let icon_path = toml_path.parent().unwrap().join("pack.png");

    if icon_path.is_file() {
        if let Ok(icon_data) = fs::read(icon_path) {
            compiled.add_file("pack.png", VFile::Binary(icon_data));
        }
    }

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

    let dist_extension = if args.zip { ".zip" } else { "" };

    let dist_path = dist_path.join(project_config.pack.name + dist_extension);

    if args.check {
        print_success("Project is valid and can be built.");
    } else {
        #[cfg(feature = "zip")]
        if args.zip {
            output.zip_with_comment(
                &dist_path,
                format!(
                    "{} - v{}",
                    &project_config.pack.description, &project_config.pack.version
                ),
            )?;
        } else {
            output.place(&dist_path)?;
        }

        #[cfg(not(feature = "zip"))]
        output.place(&dist_path)?;

        print_success(format!(
            "Finished building{and_package_msg} project to {}",
            dist_path.absolutize_from(path)?.display()
        ));
    }

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
    let path = path.absolutize()?;
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
