use std::path::{Path, PathBuf};

use reqwest::Client;
use reqwest::redirect::Policy;
use walkdir::WalkDir;

use spoon_core::{NoticeEvent, ReqwestClientBuilder, SpoonEvent};

pub use spoon_core::normalize_proxy_url;

pub fn find_first_named_file(root: &Path, candidates: &[&str]) -> Option<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .find(|entry| {
            entry.file_type().is_file()
                && candidates.iter().any(|candidate| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
                })
        })
        .map(|entry| entry.into_path())
}

pub fn find_all_named_files(root: &Path, candidates: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file()
                && candidates.iter().any(|candidate| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
                })
        })
        .map(|entry| entry.into_path())
        .collect()
}

pub fn unique_existing_dirs(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in paths {
        if !path.exists() || !path.is_dir() {
            continue;
        }
        if out.iter().any(|existing| existing == &path) {
            continue;
        }
        out.push(path);
    }
    out
}

pub fn path_components_lowercase(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect()
}

pub fn join_windows_path(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(";")
}

pub fn emit_notice(emit: Option<&spoon_core::EventSender>, message: &str) {
    tracing::info!("{message}");
    if let Some(sender) = emit {
        sender.send(SpoonEvent::Notice(NoticeEvent::info(message)));
    }
}

pub fn http_client(proxy: &str) -> spoon_core::Result<Client> {
    http_client_with_redirect(proxy, Policy::limited(10))
}

pub fn http_client_with_redirect(proxy: &str, policy: Policy) -> spoon_core::Result<Client> {
    ReqwestClientBuilder::new()
        .proxy(proxy)?
        .redirect_policy(policy)
        .build()
        .map_err(|err| err.context("failed to build MSVC HTTP client"))
}
