use serde::Serialize;

use std::path::{Path, PathBuf};

use spoon_backend::{
    msvc::{clear_cache as clear_msvc_cache, msvc_cache_root, prune_cache as prune_msvc_cache},
    scoop::{clear_cache as clear_scoop_cache, prune_cache as prune_scoop_cache, scoop_cache_root},
};

use super::{CommandResult, CommandStatus, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheScope {
    All,
    Scoop,
    Msvc,
}

#[derive(Debug, Clone, Serialize)]
pub struct CachePaths {
    pub scoop: Option<String>,
    pub msvc: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheActionOutcome {
    pub kind: &'static str,
    pub action: &'static str,
    pub scope: &'static str,
    pub success: bool,
    pub title: String,
    pub streamed: bool,
    pub output: Vec<String>,
    pub paths: CachePaths,
}

#[derive(Debug, Clone)]
pub struct CacheRoots {
    pub scoop: PathBuf,
    pub msvc: PathBuf,
}

pub fn roots_for_tool_root(tool_root: &Path) -> CacheRoots {
    CacheRoots {
        scoop: scoop_cache_root(tool_root),
        msvc: msvc_cache_root(tool_root),
    }
}

pub fn prune_for_tool_root(tool_root: &Path, scope: CacheScope) -> Result<CommandResult> {
    prune(&roots_for_tool_root(tool_root), scope)
}

pub fn clear_for_tool_root(tool_root: &Path, scope: CacheScope) -> Result<CommandResult> {
    clear(&roots_for_tool_root(tool_root), scope)
}

pub fn action_result_for_tool_root(
    tool_root: &Path,
    scope: CacheScope,
    action: &'static str,
    result: &CommandResult,
) -> CacheActionOutcome {
    action_result(&roots_for_tool_root(tool_root), scope, action, result)
}

pub fn prune(roots: &CacheRoots, scope: CacheScope) -> Result<CommandResult> {
    let mut output = Vec::new();
    match scope {
        CacheScope::All => {
            output.extend(prune_scoop_cache(&roots.scoop)?);
            output.extend(prune_msvc_cache(&roots.msvc)?);
        }
        CacheScope::Scoop => output.extend(prune_scoop_cache(&roots.scoop)?),
        CacheScope::Msvc => output.extend(prune_msvc_cache(&roots.msvc)?),
    }
    Ok(CommandResult {
        title: "prune cache".to_string(),
        status: CommandStatus::Success,
        output,
        streamed: false,
    })
}

pub fn clear(roots: &CacheRoots, scope: CacheScope) -> Result<CommandResult> {
    let mut output = Vec::new();
    match scope {
        CacheScope::All => {
            output.extend(clear_scoop_cache(&roots.scoop)?);
            output.extend(clear_msvc_cache(&roots.msvc)?);
        }
        CacheScope::Scoop => output.extend(clear_scoop_cache(&roots.scoop)?),
        CacheScope::Msvc => output.extend(clear_msvc_cache(&roots.msvc)?),
    }
    Ok(CommandResult {
        title: "clear cache".to_string(),
        status: CommandStatus::Success,
        output,
        streamed: false,
    })
}

pub fn action_result(
    roots: &CacheRoots,
    scope: CacheScope,
    action: &'static str,
    result: &CommandResult,
) -> CacheActionOutcome {
    CacheActionOutcome {
        kind: "cache_action",
        action,
        scope: scope_label(scope),
        success: result.is_success(),
        title: result.title.clone(),
        streamed: result.streamed,
        output: result.output.clone(),
        paths: cache_paths(roots, scope),
    }
}

fn scope_label(scope: CacheScope) -> &'static str {
    match scope {
        CacheScope::All => "all",
        CacheScope::Scoop => "scoop",
        CacheScope::Msvc => "msvc",
    }
}

fn cache_paths(roots: &CacheRoots, scope: CacheScope) -> CachePaths {
    match scope {
        CacheScope::All => CachePaths {
            scoop: Some(roots.scoop.display().to_string()),
            msvc: Some(roots.msvc.display().to_string()),
        },
        CacheScope::Scoop => CachePaths {
            scoop: Some(roots.scoop.display().to_string()),
            msvc: None,
        },
        CacheScope::Msvc => CachePaths {
            scoop: None,
            msvc: Some(roots.msvc.display().to_string()),
        },
    }
}
