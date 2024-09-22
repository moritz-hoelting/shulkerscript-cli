use anyhow::Result;
use path_absolutize::Absolutize as _;
use shulkerbox::virtual_fs::{VFile, VFolder};
use std::{
    borrow::Cow,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{
    terminal_output::{print_error, print_info, print_success},
    util::Relativize as _,
};

#[derive(Debug, clap::Args, Clone)]
#[command(allow_missing_positional = true)]
pub struct MigrateArgs {
    /// The path of the project to migrate.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// The path of the folder to create the ShulkerScript project.
    pub target: PathBuf,
    /// Force migration even if some features will be lost.
    #[arg(short, long)]
    pub force: bool,
}

pub fn migrate(args: &MigrateArgs) -> Result<()> {
    let base_path = args.path.as_path();
    let base_path = if base_path.is_absolute() {
        Cow::Borrowed(base_path)
    } else {
        base_path.absolutize().unwrap_or(Cow::Borrowed(base_path))
    }
    .ancestors()
    .find(|p| p.join("pack.mcmeta").exists())
    .map(|p| p.relativize().unwrap_or_else(|| p.to_path_buf()));

    if let Some(base_path) = base_path {
        print_info(format!(
            "Migrating from {:?} to {:?}",
            base_path, args.target
        ));

        let mcmeta_path = base_path.join("pack.mcmeta");
        let mcmeta: serde_json::Value =
            serde_json::from_reader(BufReader::new(fs::File::open(&mcmeta_path)?))?;

        if !args.force && !is_mcmeta_compatible(&mcmeta) {
            print_error("Your datapack uses features in the pack.mcmeta file that are not yet supported by ShulkerScript.");
            print_error(
                r#""features", "filter", "overlays" and "language" will get lost if you continue."#,
            );
            print_error("Use the force flag to continue anyway.");

            return Err(anyhow::anyhow!("Incompatible mcmeta."));
        }

        let mcmeta = serde_json::from_value::<McMeta>(mcmeta)?;

        let mut root = VFolder::new();
        root.add_file("pack.toml", generate_pack_toml(&base_path, &mcmeta)?);

        let data_path = base_path.join("data");
        if data_path.exists() && data_path.is_dir() {
            for namespace in data_path.read_dir()? {
                let namespace = namespace?;
                if namespace.file_type()?.is_dir() {
                    handle_namespace(&mut root, &namespace.path())?;
                }
            }
        } else {
            print_error("Could not find a data folder.");
        }

        root.place(&args.target)?;

        print_success("Migration successful.");
        Ok(())
    } else {
        let msg = format!(
            "Could not find a valid datapack to migrate at {}.",
            args.path.display()
        );
        print_error(&msg);
        Err(anyhow::anyhow!("{}", &msg))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct McMeta {
    pack: McMetaPack,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct McMetaPack {
    description: String,
    pack_format: u8,
}

fn is_mcmeta_compatible(mcmeta: &serde_json::Value) -> bool {
    mcmeta.as_object().map_or(false, |mcmeta| {
        mcmeta.len() == 1
            && mcmeta.contains_key("pack")
            && mcmeta["pack"]
                .as_object()
                .is_some_and(|pack| !pack.contains_key("supported_formats"))
    })
}

fn generate_pack_toml(base_path: &Path, mcmeta: &McMeta) -> Result<VFile> {
    // check if there are any directories in namespaces other than `functions`, `function` and `tags`
    let mut err = false;
    let requires_assets_dir = base_path.join("data").read_dir()?.any(|entry_i| {
        if let Ok(entry_i) = entry_i {
            if let Ok(metadata_i) = entry_i.metadata() {
                metadata_i.is_dir()
                    && entry_i
                        .path()
                        .read_dir()
                        .map(|mut dir| {
                            dir.any(|entry_ii| {
                                if let Ok(entry_ii) = entry_ii {
                                    ["functions", "function", "tags"]
                                        .contains(&entry_ii.file_name().to_string_lossy().as_ref())
                                } else {
                                    err = true;
                                    true
                                }
                            })
                        })
                        .map_err(|e| {
                            err = true;
                            e
                        })
                        .unwrap_or_default()
            } else {
                err = true;
                true
            }
        } else {
            err = true;
            true
        }
    });

    if err {
        print_error("Error reading data directory");
        return Err(anyhow::anyhow!("Error reading data directory"));
    }

    let assets_dir_fragment = requires_assets_dir.then(|| {
        toml::toml! {
            [compiler]
            assets = "./assets"
        }
    });

    let name = base_path
        .absolutize()?
        .file_name()
        .expect("No file name")
        .to_string_lossy()
        .into_owned();
    let description = mcmeta.pack.description.as_str();
    let pack_format = mcmeta.pack.pack_format;

    let main_fragment = toml::toml! {
        [pack]
        name = name
        description = description
        format = pack_format
        version = "0.1.0"
    };

    let assets_dir_fragment_text = assets_dir_fragment
        .map(|fragment| toml::to_string_pretty(&fragment))
        .transpose()?;

    // stringify the toml fragments and add them to the pack.toml file
    toml::to_string_pretty(&main_fragment)
        .map(|mut text| {
            if let Some(assets_dir_fragment_text) = assets_dir_fragment_text {
                text.push('\n');
                text.push_str(&assets_dir_fragment_text);
            }
            VFile::Text(text)
        })
        .map_err(|e| e.into())
}

fn handle_namespace(root: &mut VFolder, namespace: &Path) -> Result<()> {
    let namespace_name = namespace
        .file_name()
        .expect("path cannot end with ..")
        .to_string_lossy();

    // migrate all subfolders of namespace
    for subfolder in namespace.read_dir()? {
        let subfolder = subfolder?;
        if !subfolder.file_type()?.is_dir() {
            continue;
        }

        let filename = subfolder.file_name();
        let filename = filename.to_string_lossy();

        if ["function", "functions"].contains(&filename.as_ref()) {
            // migrate functions
            for entry in WalkDir::new(subfolder.path()).min_depth(1) {
                let entry = entry?;
                if entry.file_type().is_file()
                    && entry.path().extension().unwrap_or_default() == "mcfunction"
                {
                    handle_function(root, namespace, &namespace_name, entry.path())?;
                }
            }
        } else if filename.as_ref() == "tags" {
            // migrate tags
            for tag_type in subfolder.path().read_dir()? {
                handle_tag_type_dir(root, &namespace_name, &tag_type?.path())?;
            }
        } else {
            // copy all other files to the asset folder
            let vfolder = VFolder::try_from(subfolder.path().as_path())?;
            root.add_existing_folder(&format!("assets/data/{namespace_name}/{filename}"), vfolder);
        }
    }

    Ok(())
}

fn handle_function(
    root: &mut VFolder,
    namespace: &Path,
    namespace_name: &str,
    function: &Path,
) -> Result<()> {
    let function_path = pathdiff::diff_paths(function, namespace.join("function"))
        .expect("function path is always a subpath of namespace/function")
        .to_string_lossy()
        .replace('\\', "/");
    let function_path = function_path
        .trim_start_matches("./")
        .trim_end_matches(".mcfunction");

    // indent lines and prefix comments with `///` and commands with `/`
    let content = fs::read_to_string(function)?
        .lines()
        .map(|l| {
            if l.trim_start().starts_with('#') {
                format!("    {}", l.replacen('#', "///", 1))
            } else if l.is_empty() {
                String::new()
            } else {
                format!("    /{}", l)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let function_name = function_path
        .split('/')
        .last()
        .expect("split always returns at least one element")
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_");

    // generate the full content of the function file
    let full_content = indoc::formatdoc!(
        r#"// This file was automatically migrated by ShulkerScript CLI v{version} from file "{function}"
        namespace "{namespace_name}";

        #[deobfuscate = "{function_path}"]
        fn {function_name}() {{
        {content}
        }}
        "#,
        version = env!("CARGO_PKG_VERSION"),
        function = function.display()
    );

    root.add_file(
        &format!("src/functions/{namespace_name}/{function_path}.shu"),
        VFile::Text(full_content),
    );

    Ok(())
}

fn handle_tag_type_dir(root: &mut VFolder, namespace: &str, tag_type_dir: &Path) -> Result<()> {
    let tag_type = tag_type_dir
        .file_name()
        .expect("cannot end with ..")
        .to_string_lossy();

    // loop through all tag files in the tag type directory
    for entry in WalkDir::new(tag_type_dir).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "json" {
            handle_tag(root, namespace, tag_type_dir, &tag_type, entry.path())?;
        }
    }

    Ok(())
}

fn handle_tag(
    root: &mut VFolder,
    namespace: &str,
    tag_type_dir: &Path,
    tag_type: &str,
    tag: &Path,
) -> Result<()> {
    let tag_path = pathdiff::diff_paths(tag, tag_type_dir)
        .expect("tag path is always a subpath of tag_type_dir")
        .to_string_lossy()
        .replace('\\', "/");
    let tag_path = tag_path.trim_start_matches("./").trim_end_matches(".json");

    if let Ok(content) = serde_json::from_reader::<_, Tag>(BufReader::new(File::open(tag)?)) {
        // generate "of <type>" if the tag type is not "function"
        let of_type = if tag_type == "function" {
            String::new()
        } else {
            format!(r#" of "{tag_type}""#)
        };

        let replace = if content.replace { " replace" } else { "" };

        // indent, quote and join the values
        let values = content
            .values
            .iter()
            .map(|t| format!(r#"    "{t}""#))
            .collect::<Vec<_>>()
            .join(",\n");

        let generated = indoc::formatdoc!(
            r#"// This file was automatically migrated by ShulkerScript CLI v{version} from file "{tag}"
            namespace "{namespace}";

            tag "{tag_path}"{of_type}{replace} [
            {values}
            ]
            "#,
            version = env!("CARGO_PKG_VERSION"),
            tag = tag.display(),
        );

        root.add_file(
            &format!("src/tags/{namespace}/{tag_type}/{tag_path}.shu"),
            VFile::Text(generated),
        );

        Ok(())
    } else {
        print_error(format!(
            "Could not read tag file at {}. Required attribute of entries is not yet supported",
            tag.display()
        ));
        Err(anyhow::anyhow!(
            "Could not read tag file at {}",
            tag.display()
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct Tag {
    #[serde(default)]
    replace: bool,
    values: Vec<String>,
}
