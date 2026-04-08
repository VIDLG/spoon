pub fn config_root_set(root: &str) {
    tracing::info!(root = root, "config.root.set");
}

pub fn config_root_unset() {
    tracing::info!("config.root.unset");
}

pub fn config_target_open_in_editor(target: &str, path: &str, command: &str) {
    tracing::info!(
        target = target,
        path = path,
        command = command,
        "config.target.open_in_editor"
    );
}

pub fn config_target_open_in_explorer(target: &str, path: &str) {
    tracing::info!(
        target = target,
        path = path,
        "config.target.open_in_explorer"
    );
}
