use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct GitConfig {
    pub user_name: String,
    pub user_email: String,
    pub default_branch: String,
    pub proxy: String,
}

#[derive(Debug, Clone, Default)]
pub struct ClaudeConfig {
    pub base_url: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Default)]
pub struct CodexConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub editor: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub root: String,
    #[serde(default = "default_msvc_arch")]
    pub msvc_arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PolicyConfig {
    #[serde(default)]
    pub python: PythonPolicyConfig,
    #[serde(default)]
    pub git: GitPolicyConfig,
    #[serde(default)]
    pub msvc: MsvcPolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PythonPolicyConfig {
    #[serde(default)]
    pub pip_mirror: String,
    #[serde(default = "default_python_command_profile")]
    pub command_profile: String,
}

impl Default for PythonPolicyConfig {
    fn default() -> Self {
        Self {
            pip_mirror: String::new(),
            command_profile: default_python_command_profile(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitPolicyConfig {
    #[serde(default)]
    pub follow_spoon_proxy: bool,
    #[serde(default = "default_git_command_profile")]
    pub command_profile: String,
}

impl Default for GitPolicyConfig {
    fn default() -> Self {
        Self {
            follow_spoon_proxy: false,
            command_profile: default_git_command_profile(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsvcPolicyConfig {
    #[serde(default = "default_msvc_command_profile")]
    pub command_profile: String,
}

impl Default for MsvcPolicyConfig {
    fn default() -> Self {
        Self {
            command_profile: default_msvc_command_profile(),
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            editor: String::new(),
            proxy: String::new(),
            root: String::new(),
            msvc_arch: default_msvc_arch(),
        }
    }
}

fn default_msvc_arch() -> String {
    "auto".to_string()
}

fn default_python_command_profile() -> String {
    "default".to_string()
}

fn default_git_command_profile() -> String {
    "default".to_string()
}

fn default_msvc_command_profile() -> String {
    "default".to_string()
}
