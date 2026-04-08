use std::fs;

use anyhow::{Context, Result};
use ini::Ini;
use serde_json::{Map, Value};
use toml_edit::{DocumentMut, Item, Table, value};

use super::model::{ClaudeConfig, CodexConfig, GitConfig, GlobalConfig, PolicyConfig};
use super::paths::{
    claude_settings_path, codex_auth_path, codex_config_path, git_config_path, global_config_path,
};

pub fn load_git_config() -> GitConfig {
    let ini = load_ini_file(git_config_path());
    let http_proxy = ini_value(&ini, Some("http"), "proxy").unwrap_or_default();
    let https_proxy = ini_value(&ini, Some("https"), "proxy").unwrap_or_default();
    GitConfig {
        user_name: ini_value(&ini, Some("user"), "name").unwrap_or_default(),
        user_email: ini_value(&ini, Some("user"), "email").unwrap_or_default(),
        default_branch: ini_value(&ini, Some("init"), "defaultBranch")
            .unwrap_or_else(|| "main".to_string()),
        proxy: if !https_proxy.trim().is_empty() {
            https_proxy
        } else {
            http_proxy
        },
    }
}

pub fn save_git_config(config: &GitConfig) -> Result<()> {
    let path = git_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut ini = load_ini_file(&path);
    ini.with_section(Some("init"))
        .set("defaultBranch", config.default_branch.clone());
    set_ini_value(&mut ini, Some("user"), "name", &config.user_name);
    set_ini_value(&mut ini, Some("user"), "email", &config.user_email);
    set_ini_value(&mut ini, Some("http"), "proxy", &config.proxy);
    set_ini_value(&mut ini, Some("https"), "proxy", &config.proxy);
    ini.write_to_file(&path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load_claude_config() -> ClaudeConfig {
    let mut cfg = ClaudeConfig::default();
    let settings_path = claude_settings_path();
    if let Ok(content) = fs::read_to_string(&settings_path)
        && let Ok(json) = serde_json::from_str::<Value>(&content)
        && let Some(env) = json.get("env")
    {
        cfg.base_url = env
            .get("ANTHROPIC_BASE_URL")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        cfg.auth_token = env
            .get("ANTHROPIC_AUTH_TOKEN")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }

    if cfg.base_url.is_empty() {
        cfg.base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    }
    if cfg.auth_token.is_empty() {
        cfg.auth_token = std::env::var("ANTHROPIC_AUTH_TOKEN").unwrap_or_default();
    }
    cfg
}

pub fn save_claude_config(config: &ClaudeConfig) -> Result<()> {
    let path = claude_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut root = if path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_else(|_| Value::Object(Map::new()))
    } else {
        Value::Object(Map::new())
    };

    if !root.is_object() {
        root = Value::Object(Map::new());
    }
    let obj = root
        .as_object_mut()
        .context("expected JSON object in Claude settings")?;
    if !obj.contains_key("env") || !obj.get("env").is_some_and(Value::is_object) {
        obj.insert("env".into(), Value::Object(Map::new()));
    }
    let env = obj
        .get_mut("env")
        .and_then(Value::as_object_mut)
        .context("expected env object in Claude settings")?;
    env.insert(
        "ANTHROPIC_BASE_URL".into(),
        Value::String(config.base_url.clone()),
    );
    env.insert(
        "ANTHROPIC_AUTH_TOKEN".into(),
        Value::String(config.auth_token.clone()),
    );

    fs::write(&path, serde_json::to_string_pretty(&root)?)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load_codex_config(default_model: &str) -> CodexConfig {
    let mut cfg = CodexConfig {
        base_url: "https://api.openai.com".to_string(),
        model: default_model.to_string(),
        ..Default::default()
    };

    let config_path = codex_config_path();
    if let Ok(content) = fs::read_to_string(&config_path)
        && let Ok(doc) = content.parse::<DocumentMut>()
    {
        if let Some(model) = doc.get("model").and_then(|item| item.as_str()) {
            cfg.model = model.to_string();
        }
        if let Some(section) = doc
            .get("model_providers")
            .and_then(|item| item.get("OpenAI"))
            && let Some(base_url) = section.get("base_url").and_then(|item| item.as_str())
        {
            cfg.base_url = base_url.to_string();
        }
    }

    let auth_path = codex_auth_path();
    if let Ok(content) = fs::read_to_string(&auth_path)
        && let Ok(json) = serde_json::from_str::<Value>(&content)
    {
        cfg.api_key = json
            .get("OPENAI_API_KEY")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }

    if cfg.api_key.is_empty() {
        cfg.api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    }
    if cfg.base_url == "https://api.openai.com"
        && let Ok(base_url) = std::env::var("OPENAI_BASE_URL")
        && !base_url.trim().is_empty()
    {
        cfg.base_url = base_url;
    }
    cfg
}

pub fn save_codex_config(config: &CodexConfig) -> Result<()> {
    let config_path = codex_config_path();
    let auth_path = codex_auth_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut doc = DocumentMut::new();
    doc["model_provider"] = value("OpenAI");
    doc["model"] = value(config.model.clone());
    doc["review_model"] = value(config.model.clone());
    doc["model_reasoning_effort"] = value("medium");
    doc["disable_response_storage"] = value(true);
    doc["network_access"] = value("enabled");
    doc["model_context_window"] = value(400000);
    doc["model_auto_compact_token_limit"] = value(300000);
    doc["model_providers"]["OpenAI"]["name"] = value("OpenAI");
    doc["model_providers"]["OpenAI"]["base_url"] = value(config.base_url.clone());
    doc["model_providers"]["OpenAI"]["wire_api"] = value("responses");

    fs::write(&config_path, doc.to_string())
        .with_context(|| format!("failed to write {}", config_path.display()))?;
    let auth_json = serde_json::json!({ "OPENAI_API_KEY": config.api_key });
    fs::write(&auth_path, serde_json::to_string_pretty(&auth_json)?)
        .with_context(|| format!("failed to write {}", auth_path.display()))?;
    Ok(())
}

pub fn load_global_config() -> GlobalConfig {
    let path = global_config_path();
    let mut cfg = fs::read_to_string(&path)
        .ok()
        .and_then(|content| toml::from_str::<GlobalConfig>(&content).ok())
        .unwrap_or_default();
    cfg.msvc_arch = super::paths::msvc_arch_config_value(&cfg);
    cfg
}

pub fn save_global_config(config: &GlobalConfig) -> Result<()> {
    let path = global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut normalized = config.clone();
    normalized.msvc_arch = super::paths::msvc_arch_config_value(&normalized);

    let mut doc = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| content.parse::<DocumentMut>().ok())
            .unwrap_or_default()
    } else {
        DocumentMut::new()
    };
    doc["editor"] = value(normalized.editor);
    doc["proxy"] = value(normalized.proxy);
    doc["root"] = value(normalized.root);
    doc["msvc_arch"] = value(normalized.msvc_arch);
    let rendered = doc
        .to_string()
        .replace("[policy ]\r\n\r\n", "")
        .replace("[policy ]\n\n", "");
    fs::write(&path, rendered).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load_policy_config() -> PolicyConfig {
    #[derive(serde::Deserialize, Default)]
    struct PolicyWrapper {
        #[serde(default)]
        policy: PolicyConfig,
    }

    let path = global_config_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return PolicyConfig::default();
    };
    let Ok(wrapper) = toml::from_str::<PolicyWrapper>(&content) else {
        return PolicyConfig::default();
    };
    wrapper.policy
}

pub fn load_pip_index_url() -> String {
    let ini = load_ini_file(super::paths::pip_config_path());
    ini_value(&ini, Some("global"), "index-url").unwrap_or_default()
}

pub fn save_policy_config(config: &PolicyConfig) -> Result<()> {
    let path = global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut doc = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| content.parse::<DocumentMut>().ok())
            .unwrap_or_default()
    } else {
        DocumentMut::new()
    };
    if !matches!(doc.get("policy"), Some(Item::Table(_))) {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["policy"] = Item::Table(table);
    }
    if let Item::Table(table) = &mut doc["policy"] {
        table.set_implicit(true);
    }
    if !matches!(doc["policy"].get("python"), Some(Item::Table(_))) {
        doc["policy"]["python"] = Item::Table(Table::new());
    }
    if !matches!(doc["policy"].get("git"), Some(Item::Table(_))) {
        doc["policy"]["git"] = Item::Table(Table::new());
    }
    if !matches!(doc["policy"].get("msvc"), Some(Item::Table(_))) {
        doc["policy"]["msvc"] = Item::Table(Table::new());
    }

    doc["policy"]["python"]["pip_mirror"] = value(config.python.pip_mirror.clone());
    doc["policy"]["python"]["command_profile"] = value(config.python.command_profile.clone());
    doc["policy"]["git"]["follow_spoon_proxy"] = value(config.git.follow_spoon_proxy);
    doc["policy"]["git"]["command_profile"] = value(config.git.command_profile.clone());
    doc["policy"]["msvc"]["command_profile"] = value(config.msvc.command_profile.clone());
    let rendered = doc
        .to_string()
        .replace("[policy ]\r\n\r\n", "")
        .replace("[policy ]\n\n", "");
    fs::write(&path, rendered).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn load_ini_file(path: impl AsRef<std::path::Path>) -> Ini {
    Ini::load_from_file_noescape(path).unwrap_or_else(|_| Ini::new())
}

fn ini_value(ini: &Ini, section: Option<&str>, key: &str) -> Option<String> {
    ini.get_from(section, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn set_ini_value(ini: &mut Ini, section: Option<&str>, key: &str, value: &str) {
    if value.trim().is_empty() {
        ini.delete_from(section, key);
    } else {
        ini.with_section(section.map(str::to_string))
            .set(key, value.trim().to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        load_global_config, load_policy_config, save_git_config, save_global_config,
        save_policy_config,
    };
    use crate::config::{self, GitConfig, GlobalConfig, PolicyConfig};

    #[test]
    fn global_config_defaults_to_auto_arch() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-config-defaults-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());
        fs::write(
            temp_home.join(".spoon").join("config.toml"),
            "editor = \"\"\n",
        )
        .unwrap();

        let loaded = load_global_config();
        assert_eq!(loaded.msvc_arch, "auto");

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn global_config_save_normalizes_arch_values() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-config-save-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());

        save_global_config(&GlobalConfig {
            editor: String::new(),
            proxy: String::new(),
            root: "D:\\spoon".to_string(),
            msvc_arch: "x86_64".to_string(),
        })
        .unwrap();

        let content = fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
        assert!(content.contains("msvc_arch = \"x64\""), "{content}");

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn save_global_config_preserves_policy_table() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-config-policy-preserve-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());
        fs::write(
            temp_home.join(".spoon").join("config.toml"),
            "editor = \"zed\"\n[policy.python]\npip_mirror = \"tuna\"\n",
        )
        .unwrap();

        save_global_config(&GlobalConfig {
            editor: "nano".to_string(),
            proxy: String::new(),
            root: "D:\\spoon".to_string(),
            msvc_arch: "auto".to_string(),
        })
        .unwrap();

        let content = fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
        assert!(content.contains("editor = \"nano\""), "{content}");
        assert!(content.contains("pip_mirror = \"tuna\""), "{content}");
        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn policy_config_round_trips() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-policy-roundtrip-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());

        let policy = PolicyConfig {
            python: crate::config::PythonPolicyConfig {
                pip_mirror: "tuna".to_string(),
                command_profile: "extended".to_string(),
            },
            git: crate::config::GitPolicyConfig {
                follow_spoon_proxy: true,
                command_profile: "extended".to_string(),
            },
            msvc: crate::config::MsvcPolicyConfig {
                command_profile: "extended".to_string(),
            },
        };
        save_policy_config(&policy).unwrap();
        let loaded = load_policy_config();
        assert_eq!(loaded, policy);

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn git_command_profile_round_trips_inside_policy_config() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-git-command-profile-roundtrip-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());

        let policy = PolicyConfig {
            python: crate::config::PythonPolicyConfig::default(),
            git: crate::config::GitPolicyConfig {
                follow_spoon_proxy: false,
                command_profile: "extended".to_string(),
            },
            msvc: crate::config::MsvcPolicyConfig::default(),
        };
        save_policy_config(&policy).unwrap();
        let loaded = load_policy_config();
        assert_eq!(loaded.git.command_profile, "extended");

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn msvc_command_profile_round_trips_inside_policy_config() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-msvc-command-profile-roundtrip-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());

        let policy = PolicyConfig {
            python: crate::config::PythonPolicyConfig::default(),
            git: crate::config::GitPolicyConfig::default(),
            msvc: crate::config::MsvcPolicyConfig {
                command_profile: "extended".to_string(),
            },
        };
        save_policy_config(&policy).unwrap();
        let loaded = load_policy_config();
        assert_eq!(loaded.msvc.command_profile, "extended");

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn save_git_config_preserves_unrelated_native_fields() {
        config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-git-config-preserve-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(temp_home.join(".spoon")).unwrap();
        config::set_home_override(temp_home.clone());
        fs::write(
            temp_home.join(".gitconfig"),
            "[core]\n\tautocrlf = false\n[credential]\n\thelper = manager\n[http]\n\tproxy = http://old\n",
        )
        .unwrap();

        save_git_config(&GitConfig {
            user_name: String::new(),
            user_email: String::new(),
            default_branch: "main".to_string(),
            proxy: "http://127.0.0.1:7897".to_string(),
        })
        .unwrap();

        let content = fs::read_to_string(temp_home.join(".gitconfig")).unwrap();
        assert!(content.contains("[core]"), "{content}");
        assert!(content.contains("autocrlf=false"), "{content}");
        assert!(content.contains("[credential]"), "{content}");
        assert!(content.contains("helper=manager"), "{content}");
        assert!(content.contains("proxy=http://127.0.0.1:7897"), "{content}");

        let _ = fs::remove_dir_all(temp_home);
    }
}
