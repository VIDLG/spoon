pub fn editor_default_set(label: &str, command: &str) {
    tracing::info!(label = label, command = command, "editor.default.set");
}

pub fn editor_default_cleared() {
    tracing::info!("editor.default.cleared");
}

pub fn editor_install_start(label: &str, package: &str) {
    tracing::info!(label = label, package = package, "editor.install.start");
}

pub fn editor_uninstall_start(label: &str, package: &str) {
    tracing::info!(label = label, package = package, "editor.uninstall.start");
}

pub fn editor_setup_default_inline(label: &str, command: &str) {
    tracing::info!(
        label = label,
        command = command,
        "editor.setup.default.inline"
    );
}

pub fn editor_setup_clear_default_inline() {
    tracing::info!("editor.setup.default.cleared.inline");
}

pub fn editor_setup_install_request(label: &str, requested_target: Option<&str>) {
    tracing::info!(
        label = label,
        requested_target = requested_target,
        "editor.setup.install.request"
    );
}

pub fn editor_setup_uninstall_request(label: &str, requested_target: Option<&str>) {
    tracing::info!(
        label = label,
        requested_target = requested_target,
        "editor.setup.uninstall.request"
    );
}
