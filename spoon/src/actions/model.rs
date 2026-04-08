#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAction {
    Install,
    Update,
    Uninstall,
}
