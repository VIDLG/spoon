#[derive(Debug, Clone)]
pub struct EditorStatus {
    pub command: String,
    pub executable: String,
    pub available: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorCandidate {
    pub label: &'static str,
    pub command: &'static str,
    pub package_name: &'static str,
}

pub const EDITOR_CANDIDATES: [EditorCandidate; 3] = [
    EditorCandidate {
        label: "Zed",
        command: "zed",
        package_name: "zed",
    },
    EditorCandidate {
        label: "VS Code",
        command: "code",
        package_name: "vscode",
    },
    EditorCandidate {
        label: "Nano",
        command: "nano",
        package_name: "nano",
    },
];
