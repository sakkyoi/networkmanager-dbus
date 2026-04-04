#[derive(Debug, Clone, Default)]
pub struct TypesPage {
    pub enums: Vec<EnumDef>,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub description: Vec<String>,
    pub values: Vec<EnumValue>,
    pub source_url: Option<String>,
}

impl EnumDef {
    pub fn is_bitflags_by_value(&self) -> bool {
        self.values.iter().any(|v| v.is_hex())
    }

    pub fn has_negative_value(&self) -> bool {
        self.values.iter().any(|v| v.has_negative_sign())
    }
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub value: String,
    pub description: Vec<String>,
}

impl EnumValue {
    pub fn is_hex(&self) -> bool {
        let trimmed = self.value.trim_start();
        trimmed.starts_with("0x") || trimmed.starts_with("0X")
    }

    pub fn has_negative_sign(&self) -> bool {
        self.value.trim_start().starts_with("-")
    }
}
