#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

#[cfg(windows)]
#[allow(dead_code)]
pub struct UserEnvGuard {
    path: Option<String>,
    scoop: Option<String>,
}

#[cfg(windows)]
#[allow(dead_code)]
impl UserEnvGuard {
    pub fn capture() -> Self {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ).ok();
        Self {
            path: env
                .as_ref()
                .and_then(|env| env.get_value::<String, _>("Path").ok()),
            scoop: env
                .as_ref()
                .and_then(|env| env.get_value::<String, _>("SCOOP").ok()),
        }
    }

    fn restore_value(env: &RegKey, name: &str, value: &Option<String>) {
        match value {
            Some(value) => {
                let _ = env.set_value(name, value);
            }
            None => {
                let _ = env.delete_value(name);
            }
        }
    }
}

#[cfg(windows)]
impl Drop for UserEnvGuard {
    fn drop(&mut self) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((env, _)) = hkcu.create_subkey("Environment") {
            Self::restore_value(&env, "Path", &self.path);
            Self::restore_value(&env, "SCOOP", &self.scoop);
        }
    }
}

#[cfg(not(windows))]
pub struct UserEnvGuard;

#[cfg(not(windows))]
impl UserEnvGuard {
    pub fn capture() -> Self {
        Self
    }
}
