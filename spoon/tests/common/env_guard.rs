#![allow(dead_code)]

pub struct EnvVarGuard {
    key: String,
    original: Option<String>,
}

impl EnvVarGuard {
    pub fn capture(key: &str) -> Self {
        Self {
            key: key.to_string(),
            original: std::env::var(key).ok(),
        }
    }

    pub fn clear(key: &str) -> Self {
        let original = std::env::var(key).ok();
        unsafe {
            std::env::remove_var(key);
        }
        Self {
            key: key.to_string(),
            original,
        }
    }

    pub fn set(key: &str, value: &str) -> Self {
        let original = std::env::var(key).ok();
        unsafe {
            std::env::set_var(key, value);
        }
        Self {
            key: key.to_string(),
            original,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(&self.key, value),
                None => std::env::remove_var(&self.key),
            }
        }
    }
}

pub struct MultiEnvGuard {
    guards: Vec<EnvVarGuard>,
}

impl MultiEnvGuard {
    pub fn capture(keys: &[&str]) -> Self {
        Self {
            guards: keys.iter().map(|k| EnvVarGuard::capture(k)).collect(),
        }
    }
}

pub struct PathGuard(EnvVarGuard);

impl PathGuard {
    pub fn without_scoop_entries() -> Self {
        let original = std::env::var("PATH").unwrap_or_default();
        let filtered = original
            .split(';')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .filter(|entry| !entry.to_ascii_lowercase().contains("scoop"))
            .collect::<Vec<_>>()
            .join(";");
        unsafe {
            std::env::set_var("PATH", filtered);
        }
        Self(EnvVarGuard {
            key: "PATH".to_string(),
            original: Some(original),
        })
    }

    pub fn empty() -> Self {
        let original = std::env::var("PATH").unwrap_or_default();
        unsafe {
            std::env::set_var("PATH", "");
        }
        Self(EnvVarGuard {
            key: "PATH".to_string(),
            original: Some(original),
        })
    }
}
