use anyhow::{Context, Result};
use std::{fs, path::Path};

use crate::model::spec::{ParsedSnapshot, SnapshotManifest, SnapshotPageKind};
use crate::parse::types::parse_types_page;

pub fn parse_snapshot(snapshot_dir: &Path) -> Result<ParsedSnapshot> {
    let manifest_path = snapshot_dir.join("manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let manifest: SnapshotManifest = serde_json::from_str(&manifest_text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let mut snapshot = ParsedSnapshot {
        types: Default::default(),
        interfaces: Vec::new(),
        object_path_families: manifest.object_path_families.clone(),
    };

    for page in &manifest.pages {
        let path = snapshot_dir.join(&page.file_name);

        match page.kind {
            SnapshotPageKind::Types => {
                let html = fs::read_to_string(&path)
                    .with_context(|| format!("failed to read {}", path.display()))?;
                snapshot.types = parse_types_page(&html, &page.source_url)?;
            }
            SnapshotPageKind::Interface => {
                todo!()
            }
            SnapshotPageKind::SpecIndex | SnapshotPageKind::Other => {}
        }
    }

    Ok(snapshot)
}
