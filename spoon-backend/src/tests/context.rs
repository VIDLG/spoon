use std::path::PathBuf;

use crate::{BackendContext, RuntimeLayout};

#[test]
fn runtime_layout_derives_from_root() {
    let root = PathBuf::from(r"D:\tools");
    let layout = RuntimeLayout::from_root(&root);

    assert_eq!(layout.root, root);
    assert_eq!(layout.shims, PathBuf::from(r"D:\tools\shims"));
    assert_eq!(layout.scoop.root, PathBuf::from(r"D:\tools\scoop"));
    assert_eq!(
        layout.scoop.state_root,
        PathBuf::from(r"D:\tools\scoop\state")
    );
    assert_eq!(
        layout.scoop.cache_root,
        PathBuf::from(r"D:\tools\scoop\cache")
    );
    assert_eq!(
        layout.msvc.managed.root,
        PathBuf::from(r"D:\tools\msvc\managed")
    );
    assert_eq!(
        layout.msvc.managed.state_root,
        PathBuf::from(r"D:\tools\msvc\managed\state")
    );
    assert_eq!(
        layout.msvc.managed.cache_root,
        PathBuf::from(r"D:\tools\msvc\managed\cache")
    );
    assert_eq!(
        layout.msvc.managed.toolchain_root,
        PathBuf::from(r"D:\tools\msvc\managed\toolchain")
    );
    assert_eq!(
        layout.msvc.official.instance_root,
        PathBuf::from(r"D:\tools\msvc\official\instance")
    );
    assert_eq!(
        layout.msvc.official.cache_root,
        PathBuf::from(r"D:\tools\msvc\official\cache")
    );
    assert_eq!(
        layout.msvc.official.state_root,
        PathBuf::from(r"D:\tools\msvc\official\state")
    );
}

#[test]
fn explicit_context_required_for_runtime_ops() {
    let context = BackendContext::new(
        PathBuf::from(r"D:\runtime"),
        Some("http://127.0.0.1:7890".to_string()),
        true,
        "arm64",
        "developer",
        (),
    );

    assert_eq!(context.root, PathBuf::from(r"D:\runtime"));
    assert_eq!(context.layout, RuntimeLayout::from_root(&context.root));
    assert_eq!(
        context.layout.scoop.state_root,
        PathBuf::from(r"D:\runtime\scoop\state")
    );
    assert_eq!(
        context.layout.msvc.official.instance_root,
        PathBuf::from(r"D:\runtime\msvc\official\instance")
    );
    assert_eq!(context.proxy.as_deref(), Some("http://127.0.0.1:7890"));
    assert!(context.test_mode);
    assert_eq!(context.msvc_target_arch, "arm64");
    assert_eq!(context.msvc_command_profile, "developer");
}
