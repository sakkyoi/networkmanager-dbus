pub mod cli;
pub mod config;
pub mod fetch;
pub mod model;
pub mod parse;
pub mod render;

use anyhow::Result;
use std::{fs, path::Path};

pub use config::{load_render_config, RenderConfig, ReprKind};
pub use fetch::spec::fetch_spec_snapshot;
pub use model::{
    spec::{
        FetchedPage,
        ObjectPathFamilyRecord,
        SnapshotManifest,
        SnapshotPageKind,
        TocInterfaceRecord,
    },
    types::{EnumDef, EnumValue},
};
pub use parse::snapshot::parse_snapshot;
pub use render::enums::render_types_module;

pub fn generate_types_from_snapshot_dir(
    snapshot_dir: &Path,
    config: &RenderConfig,
) -> Result<String> {
    let snapshot = parse_snapshot(snapshot_dir)?;
    Ok(render_types_module(&snapshot.types, config))
}

pub fn write_generated_types_from_snapshot_dir(
    snapshot_dir: &Path,
    config: &RenderConfig,
    output: &Path,
) -> Result<()> {
    let generated = generate_types_from_snapshot_dir(snapshot_dir, config)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output, generated)?;
    Ok(())
}
