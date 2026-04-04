use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::layout::RuntimeLayout;
use crate::scoop::{resolve_package_source, buckets};

pub fn helper_executable_path(tool_root: &Path, package_name: &str) -> Option<PathBuf> {
    let layout = RuntimeLayout::from_root(tool_root);
    let current_root = layout.scoop.package_current_root(package_name);
    let direct = current_root.join(format!("{package_name}.exe"));
    if direct.exists() {
        return Some(direct);
    }
    let resolved = tokio::runtime::Handle::current()
        .block_on(buckets::resolve_manifest(tool_root, package_name))?;
    let manifest = std::fs::read_to_string(&resolved.manifest_path).ok()?;
    let manifest: Value = serde_json::from_str(&manifest).ok()?;
    let source = resolve_package_source(&manifest).ok()?;
    source
        .bins
        .first()
        .map(|target| current_root.join(&target.relative_path))
        .filter(|path| path.exists())
}
