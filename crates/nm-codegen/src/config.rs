use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReprKind {
    I32,
    U32,
}

impl ReprKind {
    pub fn rust_type(self) -> &'static str {
        match self {
            ReprKind::I32 => "i32",
            ReprKind::U32 => "u32",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RenderConfig {
    #[serde(default)]
    pub repr_overrides: HashMap<String, ReprKind>,

    #[serde(default)]
    pub variant_name_overrides: HashMap<String, HashMap<String, String>>,

    #[serde(default)]
    pub enum_prefix_overrides: HashMap<String, String>,
}

pub fn load_render_config(path: &Path) -> Result<RenderConfig> {
    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;

    let config = serde_json::from_str::<RenderConfig>(&text)
        .with_context(|| format!("failed to parse config JSON: {}", path.display()))?;

    Ok(config)
}
