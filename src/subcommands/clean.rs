use std::{env, path::PathBuf};

use color_eyre::eyre::Result;
use path_absolutize::Absolutize;

use crate::terminal_output::{print_error, print_info, print_success};

#[derive(Debug, clap::Args, Clone)]
pub struct CleanArgs {
    /// The path of the project to clean.
    #[clap(default_value = ".")]
    pub path: PathBuf,
    /// The path of the directory where the compiled datapacks are placed.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    /// Clean the whole output folder
    #[clap(short, long)]
    pub all: bool,
    /// Force clean
    #[clap(short, long)]
    pub force: bool,
}

pub fn clean(verbose: bool, args: &CleanArgs) -> Result<()> {
    let path = args.path.as_path();
    let dist_path = args
        .output
        .clone()
        .or_else(|| env::var("DATAPACK_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| path.join("dist"));

    let mut delete_paths = Vec::new();

    let (project_config, _) = super::build::get_pack_config(path)?;

    if args.all {
        if args.force {
            delete_paths.push(dist_path.clone());
        } else {
            print_error("You must use the --force flag to clean the whole output folder.")
        }
    } else {
        delete_paths.push(dist_path.join(&project_config.pack.name));
        delete_paths.push(dist_path.join(project_config.pack.name + ".zip"));
    }

    print_info(&format!(
        "Cleaning project at {}",
        path.absolutize_from(path)?.display()
    ));

    for delete_path in delete_paths {
        if delete_path.exists() {
            if verbose {
                print_info(&format!("Deleting {:?}", delete_path));
            }
            if delete_path.is_file() {
                std::fs::remove_file(&delete_path)?;
            } else {
                std::fs::remove_dir_all(&delete_path)?;
            }
        }
    }

    if dist_path.is_dir()
        && dist_path.file_name().is_some_and(|s| s != "datapacks")
        && dist_path.read_dir()?.next().is_none()
    {
        if verbose {
            print_info(&format!("Deleting {:?}, as it is empty", dist_path));
        }
        std::fs::remove_dir(&dist_path)?;
    }

    print_success("Project cleaned successfully.");

    Ok(())
}
