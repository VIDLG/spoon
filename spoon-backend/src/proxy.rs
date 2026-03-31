use reqwest::Proxy;
use reqwest::redirect::Policy;
use url::Url;

use crate::BackendError;

pub fn normalize_proxy_url(proxy: &str) -> crate::Result<Option<String>> {
    let trimmed = proxy.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(None);
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };

    let parsed = Url::parse(&candidate)
        .map_err(|error| BackendError::Config(format!("invalid proxy '{trimmed}': {error}")))?;

    if parsed.host_str().is_none() {
        return Err(BackendError::Config(format!(
            "invalid proxy '{trimmed}': missing host"
        )));
    }

    let mut normalized = parsed.to_string();
    if parsed.path() == "/" && parsed.query().is_none() {
        normalized.pop();
    }
    Ok(Some(normalized))
}

pub struct ReqwestClientBuilder {
    proxy: Option<String>,
    redirect_policy: Policy,
}

impl ReqwestClientBuilder {
    pub fn new() -> Self {
        Self {
            proxy: None,
            redirect_policy: Policy::limited(10),
        }
    }

    pub fn proxy(mut self, proxy: &str) -> crate::Result<Self> {
        self.proxy = normalize_proxy_url(proxy)?;
        Ok(self)
    }

    pub fn redirect_policy(mut self, policy: Policy) -> Self {
        self.redirect_policy = policy;
        self
    }

    pub fn build(self) -> crate::Result<reqwest::Client> {
        let mut builder = reqwest::Client::builder().redirect(self.redirect_policy);
        if let Some(proxy) = self.proxy {
            let proxy = Proxy::all(&proxy).map_err(|error| {
                BackendError::Config(format!("invalid proxy '{proxy}': {error}"))
            })?;
            builder = builder.proxy(proxy);
        }
        builder
            .build()
            .map_err(|err| BackendError::http_client("build", err))
    }
}
