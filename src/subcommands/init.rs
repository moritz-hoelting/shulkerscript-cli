use std::{fs, path::Path};

use crate::{
    config::ProjectConfig,
    error::{Error, Result},
    terminal_output::{print_error, print_info, print_success},
    util::to_absolute_path,
};

pub fn init(
    verbose: bool,
    path: &Path,
    name: Option<&str>,
    description: Option<&str>,
    pack_format: Option<u8>,
    force: bool,
) -> Result<()> {
    if !path.exists() {
        print_error("The specified path does not exist.");
        Err(Error::PathNotFoundError(path.to_path_buf()))
    } else if !path.is_dir() {
        print_error("The specified path is not a directory.");
        Err(Error::NotDirectoryError(path.to_path_buf()))
    } else if !force && path.read_dir()?.next().is_some() {
        print_error("The specified directory is not empty.");
        Err(Error::NonEmptyDirectoryError(path.to_path_buf()))
    } else {
        print_info("Initializing a new Shulkerscript project...");

        // Create the pack.toml file
        create_pack_config(verbose, path, name, description, pack_format)?;

        // Create the .gitignore file
        create_gitignore(path, verbose)?;

        // Create the pack.png file
        create_pack_png(path, verbose)?;

        // Create the src directory
        let src_path = path.join("src");
        create_dir(&src_path, verbose)?;

        // Create the main.shu file
        create_main_file(path, verbose)?;

        print_success("Project initialized successfully.");

        Ok(())
    }
}

fn create_pack_config(
    verbose: bool,
    base_path: &Path,
    name: Option<&str>,
    description: Option<&str>,
    pack_format: Option<u8>,
) -> Result<()> {
    let pack_name = name.or_else(|| base_path.file_name().and_then(|os| os.to_str()));

    let path = base_path.join("pack.toml");

    // Load the default config
    let mut content = ProjectConfig::default();
    // Override the default values with the provided ones
    if let Some(name) = pack_name {
        content.pack.name = name.to_string();
    }
    if let Some(description) = description {
        content.pack.description = description.to_string();
    }
    if let Some(pack_format) = pack_format {
        content.pack.pack_format = pack_format;
    }

    fs::write(&path, toml::to_string_pretty(&content)?)?;
    if verbose {
        print_info(&format!(
            "Created pack.toml file at {}.",
            to_absolute_path(&path)?
        ));
    }
    Ok(())
}

fn create_dir(path: &Path, verbose: bool) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir(path)?;
        if verbose {
            print_info(&format!(
                "Created directory at {}.",
                to_absolute_path(path)?
            ));
        }
    }
    Ok(())
}

fn create_gitignore(path: &Path, verbose: bool) -> std::io::Result<()> {
    let gitignore = path.join(".gitignore");
    fs::write(&gitignore, "/dist\n")?;
    if verbose {
        print_info(&format!(
            "Created .gitignore file at {}.",
            to_absolute_path(&gitignore)?
        ));
    }
    Ok(())
}

fn create_pack_png(path: &Path, verbose: bool) -> std::io::Result<()> {
    let pack_png = path.join("pack.png");
    fs::write(&pack_png, include_bytes!("../../assets/default-icon.png"))?;
    if verbose {
        print_info(&format!(
            "Created pack.png file at {}.",
            to_absolute_path(&pack_png)?
        ));
    }
    Ok(())
}

fn create_main_file(path: &Path, verbose: bool) -> std::io::Result<()> {
    let main_file = path.join("src").join("main.shu");
    fs::write(&main_file, include_str!("../../assets/default-main.shu"))?;
    if verbose {
        print_info(&format!(
            "Created main.shu file at {}.",
            to_absolute_path(&main_file)?
        ));
    }
    Ok(())
}
