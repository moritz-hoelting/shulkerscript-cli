use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub pack: PackConfig,
}
impl ProjectConfig {
    pub fn new(pack: PackConfig) -> Self {
        Self { pack }
    }
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

impl PackConfig {
    pub fn new(name: &str, description: &str, pack_format: u8) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            pack_format,
            version: "0.1.0".to_string(),
        }
    }
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
