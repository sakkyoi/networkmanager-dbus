use anyhow::{anyhow, Context, Result};
use std::path::Path;
use url::Url;

use crate::fetch::snapshot::write_text_file;
use crate::model::spec::{
    FetchedPage,
    ObjectPathFamilyRecord,
    SnapshotManifest,
    SnapshotPageKind,
    TocInterfaceRecord,
};
use crate::parse::spec_index::parse_spec_toc;

pub fn fetch_spec_snapshot(root_url: &str, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let root = Url::parse(root_url).with_context(|| format!("invalid URL: {root_url}"))?;
    let root_html = fetch_url(root.as_str())?;

    write_text_file(&out_dir.join("spec.html"), &root_html)?;

    let spec_toc = parse_spec_toc(&root_html)?;
    let mut manifest = SnapshotManifest {
        root_url: root.as_str().to_string(),
        pages: vec![FetchedPage {
            title: "spec".to_string(),
            file_name: "spec.html".to_string(),
            source_url: root.as_str().to_string(),
            kind: SnapshotPageKind::SpecIndex,
        }],
        object_path_families: Vec::new(),
    };

    for entry in spec_toc.entries {
        let mut family_record = ObjectPathFamilyRecord {
            label: entry.object_path_label.clone(),
            path_pattern: extract_object_path_pattern(&entry.object_path_label),
            interfaces: Vec::new(),
        };

        for iface in entry.interfaces {
            if iface.href.is_empty() {
                continue;
            }

            let url = root
                .join(&iface.href)
                .with_context(|| format!("failed to join href {}", iface.href))?;

            let file_name = file_name_from_url(&url)
                .ok_or_else(|| anyhow!("cannot derive file name from {}", url))?;

            let html = fetch_url(url.as_str())?;
            write_text_file(&out_dir.join(&file_name), &html)?;

            let kind = classify_toc_interface(&iface.name);

            manifest.pages.push(FetchedPage {
                title: iface.name.clone(),
                file_name: file_name.clone(),
                source_url: url.to_string(),
                kind,
            });

            family_record.interfaces.push(TocInterfaceRecord {
                name: iface.name,
                href: iface.href,
                file_name,
                description: iface.description,
            });
        }

        manifest.object_path_families.push(family_record);
    }

    manifest.pages.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    manifest.pages.dedup_by(|a, b| a.file_name == b.file_name);

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    write_text_file(&out_dir.join("manifest.json"), &manifest_json)?;

    Ok(())
}

fn fetch_url(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("failed to GET {url}"))?
        .error_for_status()
        .with_context(|| format!("non-success status for {url}"))?;

    response
        .text()
        .with_context(|| format!("failed to read response body from {url}"))
}

fn file_name_from_url(url: &Url) -> Option<String> {
    let path = url.path();
    let last = path.split("/").filter(|s| !s.is_empty()).last()?;
    Some(last.to_string())
}


fn classify_toc_interface(title: &str) -> SnapshotPageKind {
    if title.ends_with("Types") {
        SnapshotPageKind::Types
    } else if title.starts_with("org.freedesktop.") {
        SnapshotPageKind::Interface
    } else {
        SnapshotPageKind::Other
    }
}

fn extract_object_path_pattern(label: &str) -> Option<String> {
    let marker = "The ";
    let suffix = " objects";

    if let Some(rest) = label.strip_prefix(marker) {
        if let Some(path) = rest.strip_suffix(suffix) {
            let trimmed = path.trim();
            // Although the suffix shows it's a patterned path, we still check for the ending *
            if trimmed.starts_with("/") && trimmed.ends_with("*") {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}