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
            let key_env = cfg
                .api_key_env
                .clone()
                .unwrap_or_else(|| "OPENAI_API_KEY".to_string());
            let api_key = std::env::var(&key_env).map_err(|_| {
                anyhow::anyhow!("missing API key: environment variable {key_env} is not set")
            })?;
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
