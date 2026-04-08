pub use crate::config::{
    GitPolicyConfig, GlobalConfig, MsvcPolicyConfig, PolicyConfig, PythonPolicyConfig,
    configured_tool_root, enable_test_mode, ensure_global_config_exists, ensure_process_path_entry,
    ensure_user_path_entry, git_config_path, global_config_path, home_dir, load_global_config,
    load_policy_config, msvc_arch_config_value, msvc_arch_from_config, native_msvc_arch,
    proxy_env_pairs, remove_process_path_entry, remove_user_path_entry,
    save_global_config, set_home_override, spoon_home_dir, test_mode_enabled,
};

pub use crate::packages::{ConfigEntry, desired_policy_entries};

