use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliKind {
    Plain,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CliEntry {
    Header { level: u8, title: String },
    Line { kind: CliKind, text: String },
    KeyValue { key: String, value: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CliResponse {
    pub entries: Vec<CliEntry>,
}

impl CliResponse {
    pub fn new(entries: Vec<CliEntry>) -> Self {
        Self { entries }
    }

    pub fn line(kind: CliKind, text: impl Into<String>) -> Self {
        Self::new(vec![CliEntry::Line {
            kind,
            text: text.into(),
        }])
    }
    pub fn section(title: impl Into<String>) -> CliEntry {
        CliEntry::Header {
            level: 0,
            title: title.into(),
        }
    }

    pub fn subsection(level: u8, title: impl Into<String>) -> CliEntry {
        CliEntry::Header {
            level,
            title: title.into(),
        }
    }
}
