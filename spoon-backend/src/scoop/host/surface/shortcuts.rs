use std::path::{Path, PathBuf};

use mslnk::ShellLink;
use tokio::fs;

use crate::Result;
use crate::{BackendError, BackendEvent};
use crate::scoop::{ResolvedPackageSource, ShortcutEntry};

fn start_menu_shortcuts_root(test_mode: bool) -> Result<PathBuf> {
    if test_mode || std::env::var_os("SPOON_TEST_HOME").is_some() {
        let test_home = std::env::var_os("SPOON_TEST_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("spoon-test-home"));
        return Ok(test_home.join(".spoon-test-startmenu").join("Spoon Apps"));
    }
    let data_dir = dirs::data_dir().ok_or(BackendError::PlatformDirectoryUnavailable {
        directory_label: "Windows data directory",
    })?;
    Ok(data_dir
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Spoon Apps"))
}

fn substitute_shortcut_text(value: &str, install_root: &Path, persist_root: &Path) -> String {
    value
        .replace("$dir", &install_root.display().to_string())
        .replace("$persist_dir", &persist_root.display().to_string())
        .replace("$original_dir", &install_root.display().to_string())
}

pub(crate) async fn write_shortcuts(
    install_root: &Path,
    persist_root: &Path,
    source: &ResolvedPackageSource,
    test_mode: bool,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<ShortcutEntry>> {
    if source.shortcuts.is_empty() {
        return Ok(Vec::new());
    }
    let shortcuts_root = start_menu_shortcuts_root(test_mode)?;
    fs::create_dir_all(&shortcuts_root)
        .await
        .map_err(|err| BackendError::fs("create", &shortcuts_root, err))?;

    let mut created = Vec::new();
    for shortcut in &source.shortcuts {
        let target = install_root.join(&shortcut.target_path);
        if !target.exists() {
            tracing::warn!(
                "Skipped shortcut '{}' because target was missing: {}",
                shortcut.name,
                target.display()
            );
            continue;
        }
        let shortcut_path = shortcuts_root.join(&shortcut.name).with_extension("lnk");
        if let Some(parent) = shortcut_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| BackendError::fs("create", parent, err))?;
        }
        let args = shortcut
            .args
            .as_ref()
            .map(|value| substitute_shortcut_text(value, install_root, persist_root));
        let icon = shortcut
            .icon_path
            .as_ref()
            .map(|value| install_root.join(value));
        if let Some(icon_path) = &icon && !icon_path.exists() {
            tracing::warn!(
                "Skipped shortcut icon for '{}' because it was missing: {}",
                shortcut.name,
                icon_path.display()
            );
        }
        let target_owned = target.clone();
        let shortcut_path_owned = shortcut_path.clone();
        let args_owned = args.clone();
        let icon_owned = icon.clone().filter(|path| path.exists());
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut link = ShellLink::new(&target_owned).map_err(|err| {
                BackendError::external(
                    format!(
                        "failed to create shortcut target for {}",
                        target_owned.display()
                    ),
                    err,
                )
            })?;
            if let Some(arguments) = args_owned {
                link.set_arguments(Some(arguments));
            }
            if let Some(icon_path) = icon_owned {
                link.set_icon_location(Some(icon_path.display().to_string()));
            }
            link.create_lnk(&shortcut_path_owned).map_err(|err| {
                BackendError::external(
                    format!("failed to write {}", shortcut_path_owned.display()),
                    err,
                )
            })?;
            Ok(())
        })
        .await
        .map_err(|err| BackendError::external("shortcut creation join failed", err))??;
        tracing::info!(
            "Created shortcut '{}': {}",
            shortcut.name,
            shortcut_path.display()
        );
        created.push(shortcut.clone());
    }
    Ok(created)
}

pub async fn remove_shortcuts(entries: &[ShortcutEntry], test_mode: bool) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let shortcuts_root = start_menu_shortcuts_root(test_mode)?;
    for shortcut in entries {
        let path = shortcuts_root.join(&shortcut.name).with_extension("lnk");
        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|err| BackendError::fs("remove", &path, err))?;
        }
    }
    Ok(())
}
