use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub pack: PackConfig,
    pub compiler: Option<CompilerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackConfig {
    pub name: String,
    pub description: String,
    #[serde(
        rename = "format",
        alias = "pack_format",
        default = "default_pack_format"
    )]
    pub pack_format: u8,
    pub version: String,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            name: "shulkerscript-pack".to_string(),
            description: "A Minecraft datapack created with shulkerscript".to_string(),
            pack_format: 26,
            version: "0.1.0".to_string(),
        }
    }
}

fn default_pack_format() -> u8 {
    26
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// The path of a folder which files and subfolders will be copied to the root of the datapack.
    pub assets: Option<PathBuf>,
}
