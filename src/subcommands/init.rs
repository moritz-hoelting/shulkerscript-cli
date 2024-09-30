use std::{
    borrow::Cow,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::ValueEnum;
use git2::{
    IndexAddOption as GitIndexAddOption, Repository as GitRepository, Signature as GitSignature,
};
use inquire::validator::Validation;
use path_absolutize::Absolutize;

use crate::{
    config::{PackConfig, ProjectConfig},
    error::Error,
    terminal_output::{print_error, print_info, print_success},
};

#[derive(Debug, clap::Args, Clone)]
pub struct InitArgs {
    /// The path of the folder to initialize in.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// The name of the project.
    #[arg(short, long)]
    pub name: Option<String>,
    /// The description of the project.
    #[arg(short, long)]
    pub description: Option<String>,
    /// The pack format version.
    #[arg(short, long, value_name = "FORMAT", visible_alias = "format")]
    pub pack_format: Option<u8>,
    /// The path of the icon file.
    #[arg(short, long = "icon", value_name = "PATH")]
    pub icon_path: Option<PathBuf>,
    /// Force initialization even if the directory is not empty.
    #[arg(short, long)]
    pub force: bool,
    /// The version control system to initialize. [default: git]
    #[arg(long)]
    pub vcs: Option<VersionControlSystem>,
    /// Enable verbose output.
    #[arg(short, long)]
    pub verbose: bool,
    /// Enable batch mode.
    ///
    /// In batch mode, the command will not prompt the user for input and
    /// will use the default values instead if possible or fail.
    #[arg(long)]
    pub batch: bool,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum VersionControlSystem {
    #[default]
    Git,
    None,
}

impl Display for VersionControlSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionControlSystem::Git => write!(f, "git"),
            VersionControlSystem::None => write!(f, "none"),
        }
    }
}

pub fn init(args: &InitArgs) -> Result<()> {
    if args.batch {
        initialize_batch(args)
    } else {
        initialize_interactive(args)
    }
}

fn initialize_batch(args: &InitArgs) -> Result<()> {
    let verbose = args.verbose;
    let force = args.force;
    let path = args.path.as_path();
    let description = args.description.as_deref();
    let pack_format = args.pack_format;
    let vcs = args.vcs.unwrap_or(VersionControlSystem::Git);

    if !path.exists() {
        if force {
            fs::create_dir_all(path)?;
        } else {
            print_error("The specified path does not exist.");
            Err(Error::PathNotFoundError(path.to_path_buf()))?;
        }
    } else if !path.is_dir() {
        print_error("The specified path is not a directory.");
        Err(Error::NotDirectoryError(path.to_path_buf()))?;
    } else if !force && path.read_dir()?.next().is_some() {
        print_error("The specified directory is not empty.");
        Err(Error::NonEmptyDirectoryError(path.to_path_buf()))?;
    }

    let name = args
        .name
        .as_deref()
        .or_else(|| path.file_name().and_then(|os| os.to_str()));

    print_info("Initializing a new Shulkerscript project in batch mode...");

    // Create the pack.toml file
    create_pack_config(verbose, path, name, description, pack_format)?;

    // Create the pack.png file
    create_pack_png(path, args.icon_path.as_deref(), verbose)?;

    // Create the src directory
    let src_path = path.join("src");
    create_dir(&src_path, verbose)?;

    // Create the main.shu file
    create_main_file(
        path,
        &name_to_namespace(name.unwrap_or(PackConfig::DEFAULT_NAME)),
        verbose,
    )?;

    // Initialize the version control system
    initalize_vcs(path, vcs, verbose)?;

    print_success("Project initialized successfully.");

    Ok(())
}

fn initialize_interactive(args: &InitArgs) -> Result<()> {
    const ABORT_MSG: &str = "Project initialization interrupted. Aborting...";

    let verbose = args.verbose;
    let force = args.force;
    let path = args.path.as_path();
    let description = args.description.as_deref();
    let pack_format = args.pack_format;

    if !path.exists() {
        if force {
            fs::create_dir_all(path)?;
        } else {
            match inquire::Confirm::new(
                "The specified path does not exist. Do you want to create it?",
            )
            .with_default(true)
            .prompt()
            {
                Ok(true) => fs::create_dir_all(path)?,
                Ok(false) | Err(_) => {
                    print_info(ABORT_MSG);
                    return Err(inquire::InquireError::OperationCanceled.into());
                }
            }
        }
    } else if !path.is_dir() {
        print_error("The specified path is not a directory.");
        Err(Error::NotDirectoryError(path.to_path_buf()))?
    } else if !force && path.read_dir()?.next().is_some() {
        match inquire::Confirm::new(
            "The specified directory is not empty. Do you want to continue?",
        )
        .with_default(false)
        .with_help_message("This may overwrite existing files in the directory.")
        .prompt()
        {
            Ok(false) | Err(_) => {
                print_info(ABORT_MSG);
                return Err(inquire::InquireError::OperationCanceled.into());
            }
            Ok(true) => {}
        }
    }

    let mut interrupted = false;

    let name = args.name.as_deref().map(Cow::Borrowed).or_else(|| {
        let default = path
            .file_name()
            .and_then(|os| os.to_str())
            .unwrap_or(PackConfig::DEFAULT_NAME);

        match inquire::Text::new("Enter the name of the project:")
            .with_help_message("This will be the name of your datapack folder/zip file")
            .with_default(default)
            .prompt()
        {
            Ok(res) => Some(Cow::Owned(res)),
            Err(_) => {
                interrupted = true;
                None
            }
        }
        .or_else(|| {
            path.file_name()
                .and_then(|os| os.to_str().map(Cow::Borrowed))
        })
    });

    if interrupted {
        print_info(ABORT_MSG);
        return Err(inquire::InquireError::OperationCanceled.into());
    }

    let description = description.map(Cow::Borrowed).or_else(||  {
        match inquire::Text::new("Enter the description of the project:")
            .with_help_message("This will be the description of your datapack, visible in the datapack selection screen")
            .with_default(PackConfig::DEFAULT_DESCRIPTION)
            .prompt() {
                Ok(res) => Some(Cow::Owned(res)),
                Err(_) => {
                    interrupted = true;
                    None
                }
            }
    });

    if interrupted {
        print_info(ABORT_MSG);
        return Err(inquire::InquireError::OperationCanceled.into());
    }

    let pack_format = pack_format.or_else(|| {
        match inquire::Text::new("Enter the pack format:")
            .with_help_message("This will determine the Minecraft version compatible with your pack, find more on the Minecraft wiki")
            .with_default(PackConfig::DEFAULT_PACK_FORMAT.to_string().as_str())
            .with_validator(|v: &str| Ok(
                v.parse::<u8>()
                .map(|_| Validation::Valid)
                .unwrap_or(Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom("Invalid pack format".to_string())))))
            .prompt() {
                Ok(res) => res.parse().ok(),
                Err(_) => {
                    interrupted = true;
                    None
                }
            }
    });

    if interrupted {
        print_info(ABORT_MSG);
        return Err(inquire::InquireError::OperationCanceled.into());
    }

    let vcs = args.vcs.unwrap_or_else(|| {
        match inquire::Select::new(
            "Select the version control system:",
            vec![VersionControlSystem::Git, VersionControlSystem::None],
        )
        .with_help_message("This will initialize a version control system")
        .prompt()
        {
            Ok(res) => res,
            Err(_) => {
                interrupted = true;
                VersionControlSystem::Git
            }
        }
    });

    if interrupted {
        print_info(ABORT_MSG);
        return Err(inquire::InquireError::OperationCanceled.into());
    }

    let icon_path = args.icon_path.as_deref().map(Cow::Borrowed).or_else(|| {
        let autocompleter = crate::util::PathAutocomplete::new();
        match inquire::Text::new("Enter the path of the icon file:")
            .with_help_message(
                "This will be the icon of your datapack, visible in the datapack selection screen [use \"-\" for default]",
            )
            .with_autocomplete(autocompleter)
            .with_validator(|s: &str| {
                if s == "-" {
                    Ok(Validation::Valid)
                } else {
                    let path = Path::new(s);
                    if path.exists() && path.is_file() && path.extension().is_some_and(|ext| ext == "png") {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom("Invalid file path. Path must exist and point to a png".to_string()),
                        ))
                    }
                }
            })
            .with_default("-")
            .prompt()
        {
            Ok(res) if &res == "-" => None,
            Ok(res) => Some(Cow::Owned(PathBuf::from(res))),
            Err(_) => {
                interrupted = true;
                None
            }
        }
    });

    if interrupted {
        print_info(ABORT_MSG);
        return Err(inquire::InquireError::OperationCanceled.into());
    }

    print_info("Initializing a new Shulkerscript project...");

    // Create the pack.toml file
    create_pack_config(
        verbose,
        path,
        name.as_deref(),
        description.as_deref(),
        pack_format,
    )?;

    // Create the pack.png file
    create_pack_png(path, icon_path.as_deref(), verbose)?;

    // Create the src directory
    let src_path = path.join("src");
    create_dir(&src_path, verbose)?;

    // Create the main.shu file
    create_main_file(
        path,
        &name_to_namespace(&name.unwrap_or(Cow::Borrowed("shulkerscript-pack"))),
        verbose,
    )?;

    // Initialize the version control system
    initalize_vcs(path, vcs, verbose)?;

    print_success("Project initialized successfully.");

    Ok(())
}

fn create_pack_config(
    verbose: bool,
    base_path: &Path,
    name: Option<&str>,
    description: Option<&str>,
    pack_format: Option<u8>,
) -> Result<()> {
    let path = base_path.join("pack.toml");

    // Load the default config
    let mut content = ProjectConfig::default();
    // Override the default values with the provided ones
    if let Some(name) = name {
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
        print_info(format!(
            "Created pack.toml file at {}.",
            path.absolutize()?.display()
        ));
    }
    Ok(())
}

fn create_dir(path: &Path, verbose: bool) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir(path)?;
        if verbose {
            print_info(format!(
                "Created directory at {}.",
                path.absolutize()?.display()
            ));
        }
    }
    Ok(())
}

fn create_gitignore(path: &Path, verbose: bool) -> std::io::Result<()> {
    let gitignore = path.join(".gitignore");
    fs::write(&gitignore, "/dist\n")?;
    if verbose {
        print_info(format!(
            "Created .gitignore file at {}.",
            gitignore.absolutize()?.display()
        ));
    }
    Ok(())
}

fn create_pack_png(
    project_path: &Path,
    icon_path: Option<&Path>,
    verbose: bool,
) -> std::io::Result<()> {
    let pack_png = project_path.join("pack.png");
    if let Some(icon_path) = icon_path {
        fs::copy(icon_path, &pack_png)?;
        if verbose {
            print_info(format!(
                "Copied pack.png file from {} to {}.",
                icon_path.absolutize()?.display(),
                pack_png.absolutize()?.display()
            ));
        }
    } else {
        fs::write(&pack_png, include_bytes!("../../assets/default-icon.png"))?;
        if verbose {
            print_info(format!(
                "Created pack.png file at {}.",
                pack_png.absolutize()?.display()
            ));
        }
    }
    Ok(())
}

fn create_main_file(path: &Path, namespace: &str, verbose: bool) -> std::io::Result<()> {
    let main_file = path.join("src").join("main.shu");
    fs::write(
        &main_file,
        format!(
            include_str!("../../assets/default-main.shu"),
            namespace = namespace
        ),
    )?;
    if verbose {
        print_info(format!(
            "Created main.shu file at {}.",
            main_file.absolutize()?.display()
        ));
    }
    Ok(())
}

fn initalize_vcs(path: &Path, vcs: VersionControlSystem, verbose: bool) -> Result<()> {
    match vcs {
        VersionControlSystem::None => Ok(()),
        VersionControlSystem::Git => {
            if verbose {
                print_info("Initializing a new Git repository...");
            }
            // Initalize the Git repository
            let repo = GitRepository::init(path)?;
            repo.add_ignore_rule("/dist")?;

            // Create the .gitignore file
            create_gitignore(path, verbose)?;

            // Create the initial commit
            let mut index = repo.index()?;
            let oid = index.write_tree()?;
            let tree = repo.find_tree(oid)?;
            let signature = repo
                .signature()
                .unwrap_or(GitSignature::now("Shulkerscript CLI", "cli@shulkerscript")?);
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Inital commit",
                &tree,
                &[],
            )?;

            // Create the second commit with the template files
            let mut index = repo.index()?;
            index.add_all(["."].iter(), GitIndexAddOption::DEFAULT, None)?;
            index.write()?;
            let oid = index.write_tree()?;
            let tree = repo.find_tree(oid)?;
            let parent = repo.head()?.peel_to_commit()?;
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Add template files",
                &tree,
                &[&parent],
            )?;

            print_info("Initialized a new Git repository.");

            Ok(())
        }
    }
}

fn name_to_namespace(name: &str) -> String {
    const VALID_CHARS: &str = "0123456789abcdefghijklmnopqrstuvwxyz_-.";

    name.to_lowercase()
        .chars()
        .filter_map(|c| {
            if VALID_CHARS.contains(c) {
                Some(c)
            } else if c.is_ascii_uppercase() {
                Some(c.to_ascii_lowercase())
            } else if c.is_ascii_punctuation() {
                Some('-')
            } else if c.is_ascii_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}
