#[derive(Debug, Clone, Default)]
pub struct InterfaceDef {
    pub name: String,
    pub source_url: Option<String>,
    pub object_path_family_label: Option<String>,
    pub object_path_pattern: Option<String>,
    pub methods: Vec<MethodDef>,
    pub properties: Vec<PropertyDef>,
    pub signals: Vec<SignalDef>,
}

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SignalDef {
    pub name: String,
}
