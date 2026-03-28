use std::path::PathBuf;

use crate::layout::RuntimeLayout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendContext<P> {
    pub root: PathBuf,
    pub layout: RuntimeLayout,
    pub proxy: Option<String>,
    pub test_mode: bool,
    pub msvc_target_arch: String,
    pub msvc_command_profile: String,
    pub ports: P,
}

impl<P> BackendContext<P> {
    pub fn new(
        root: PathBuf,
        proxy: Option<String>,
        test_mode: bool,
        msvc_target_arch: impl Into<String>,
        msvc_command_profile: impl Into<String>,
        ports: P,
    ) -> Self {
        let layout = RuntimeLayout::from_root(&root);
        Self {
            root,
            layout,
            proxy,
            test_mode,
            msvc_target_arch: msvc_target_arch.into(),
            msvc_command_profile: msvc_command_profile.into(),
            ports,
        }
    }
}
