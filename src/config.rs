use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shulkerscript::shulkerbox;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub pack: PackConfig,
    pub compiler: Option<CompilerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackConfig {
    pub name: String,
    pub description: String,
    #[serde(rename = "format", alias = "pack_format")]
    pub pack_format: u8,
    pub version: String,
}

impl PackConfig {
    pub const DEFAULT_NAME: &'static str = "shulkerscript-pack";
    pub const DEFAULT_DESCRIPTION: &'static str = "A Minecraft datapack created with shulkerscript";
    pub const DEFAULT_PACK_FORMAT: u8 = shulkerbox::datapack::Datapack::LATEST_FORMAT;
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            name: Self::DEFAULT_NAME.to_string(),
            description: Self::DEFAULT_DESCRIPTION.to_string(),
            pack_format: Self::DEFAULT_PACK_FORMAT,
            version: "0.1.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// The path of a folder which files and subfolders will be copied to the root of the datapack.
    pub assets: Option<PathBuf>,
}
