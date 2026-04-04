use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ShimTarget {
    pub relative_path: String,
    pub alias: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistEntry {
    pub relative_path: String,
    pub store_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub target_path: String,
    pub name: String,
    pub args: Option<String>,
    pub icon_path: Option<String>,
}
