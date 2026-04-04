use serde::{Deserialize, Serialize};

use crate::model::{interface::InterfaceDef, types::TypesPage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub root_url: String,
    pub pages: Vec<FetchedPage>,
    pub object_path_families: Vec<ObjectPathFamilyRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedPage {
    pub title: String,
    pub file_name: String,
    pub source_url: String,
    pub kind: SnapshotPageKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotPageKind {
    SpecIndex,
    Types,
    Interface,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPathFamilyRecord {
    pub label: String,
    pub path_pattern: Option<String>,
    pub interfaces: Vec<TocInterfaceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocInterfaceRecord {
    pub name: String,
    pub href: String,
    pub file_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedSnapshot {
    pub types: TypesPage,
    pub interfaces: Vec<InterfaceDef>,
    pub object_path_families: Vec<ObjectPathFamilyRecord>,
}
