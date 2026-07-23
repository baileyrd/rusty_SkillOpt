use std::sync::Arc;

use skillopt_core::{BackendConfig, ChatBackend, Provider};

use crate::anthropic::AnthropicBackend;
use crate::mock::MockBackend;
use crate::openai_compat::OpenAiCompatBackend;

/// Instantiates a [`ChatBackend`] from a [`BackendConfig`], resolving the
/// API key from the environment (a provider-appropriate default variable
/// name unless `api_key_env` overrides it). The `mock` provider needs no
/// key and never touches the network.
pub fn build_backend(cfg: &BackendConfig) -> anyhow::Result<Arc<dyn ChatBackend>> {
    match cfg.provider {
        Provider::Mock => Ok(Arc::new(MockBackend::echo(cfg.model.clone()))),
        Provider::Anthropic => {
            let key_env = cfg
                .api_key_env
                .clone()
                .unwrap_or_else(|| "ANTHROPIC_API_KEY".to_string());
            let api_key = std::env::var(&key_env).map_err(|_| {
                anyhow::anyhow!("missing API key: environment variable {key_env} is not set")
            })?;
            Ok(Arc::new(AnthropicBackend::new(
                api_key,
                cfg.base_url.clone(),
                cfg.model.clone(),
                cfg.temperature,
                cfg.max_tokens,
            )))
        }
        Provider::OpenAiCompatible => {
            // If `api_key_env` is explicitly set, that variable must be
            // present (the user asked for it by name). Otherwise fall back
            // to OPENAI_API_KEY if set, or no auth at all if not - local
            // servers like Ollama don't check it, so requiring a dummy value
            // would just be friction.
            let api_key = match &cfg.api_key_env {
                Some(key_env) => Some(std::env::var(key_env).map_err(|_| {
                    anyhow::anyhow!("missing API key: environment variable {key_env} is not set")
                })?),
                None => std::env::var("OPENAI_API_KEY").ok(),
            };
            Ok(Arc::new(OpenAiCompatBackend::new(
                api_key,
                cfg.base_url.clone(),
                cfg.model.clone(),
                cfg.temperature,
                cfg.max_tokens,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillopt_core::BackendConfig;

    fn cfg(api_key_env: Option<&str>) -> BackendConfig {
        BackendConfig {
            provider: Provider::OpenAiCompatible,
            model: "llama3".into(),
            base_url: Some("http://localhost:11434/v1".into()),
            api_key_env: api_key_env.map(|s| s.to_string()),
            temperature: None,
            max_tokens: 64,
        }
    }

    // Both cases share one test (rather than two #[test] fns) because they
    // mutate the process-global OPENAI_API_KEY env var, and Rust runs tests
    // in the same process concurrently by default.
    #[test]
    fn openai_compatible_key_resolution() {
        std::env::remove_var("OPENAI_API_KEY");

        // No api_key_env and no OPENAI_API_KEY set: succeeds with no key,
        // matching a local server like Ollama that doesn't check auth.
        assert!(build_backend(&cfg(None)).is_ok());

        // OPENAI_API_KEY set with no explicit api_key_env: picked up.
        std::env::set_var("OPENAI_API_KEY", "sk-fallback");
        assert!(build_backend(&cfg(None)).is_ok());
        std::env::remove_var("OPENAI_API_KEY");

        // Explicit api_key_env pointing at an unset variable: errors rather
        // than silently proceeding with no auth, since the user named it.
        std::env::remove_var("SOME_OTHER_KEY");
        assert!(build_backend(&cfg(Some("SOME_OTHER_KEY"))).is_err());

        // Explicit api_key_env pointing at a set variable: succeeds.
        std::env::set_var("SOME_OTHER_KEY", "sk-explicit");
        assert!(build_backend(&cfg(Some("SOME_OTHER_KEY"))).is_ok());
        std::env::remove_var("SOME_OTHER_KEY");
    }
}
