use crate::model::common::Version;

#[derive(Debug, Clone, Default)]
pub struct InterfaceDef {
    pub name: String,
    pub source_url: Option<String>,
    pub description: Vec<String>,
    pub since: Option<Version>,
    pub deprecated: Option<Version>,
    pub object_path_family_label: Option<String>,
    pub object_path_pattern: Option<String>,
    pub methods: Vec<MethodDef>,
    pub properties: Vec<PropertyDef>,
    pub signals: Vec<SignalDef>,
}

#[derive(Debug, Clone, Default)]
pub struct MethodDef {
    pub name: String,
    pub source_url: Option<String>,
    pub description: Vec<String>,
    pub since: Option<Version>,
    pub deprecated: Option<Version>,
    pub inputs: Vec<ArgumentDef>,
    pub outputs: Vec<ArgumentDef>,
}

#[derive(Debug, Clone, Default)]
pub struct PropertyDef {
    pub name: String,
    pub source_url: Option<String>,
    pub description: Vec<String>,
    pub since: Option<Version>,
    pub deprecated: Option<Version>,
    pub signature: Option<String>,
    pub access: Option<PropertyAccess>,
}

#[derive(Debug, Clone, Default)]
pub struct SignalDef {
    pub name: String,
    pub source_url: Option<String>,
    pub description: Vec<String>,
    pub since: Option<Version>,
    pub deprecated: Option<Version>,
    pub args: Vec<ArgumentDef>,
}

#[derive(Debug, Clone, Default)]
pub struct ArgumentDef {
    pub name: Option<String>,
    pub signature: Option<String>,
    pub direction: Option<ArgDirection>,
    pub description: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgDirection {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyAccess {
    Read,
    ReadWrite,
    Write,
}
