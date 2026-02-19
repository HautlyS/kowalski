use kowalski_core::config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAgentConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: usize,
    #[serde(default = "default_timeout")]
    pub request_timeout: u64,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    #[serde(default = "default_max_redirects")]
    pub max_redirects: usize,
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
    #[serde(default)]
    pub debug_logging: bool,
}

fn default_max_concurrent() -> usize {
    10
}
fn default_timeout() -> u64 {
    30
}
fn default_user_agent() -> String {
    "Kowalski Agent/1.0".to_string()
}
fn default_max_redirects() -> usize {
    5
}
fn default_true() -> bool {
    true
}
fn default_system_prompt() -> String {
    "You are a helpful assistant.".to_string()
}

impl Default for TemplateAgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: default_max_concurrent(),
            request_timeout: default_timeout(),
            user_agent: default_user_agent(),
            follow_redirects: true,
            max_redirects: default_max_redirects(),
            verify_ssl: true,
            proxy: None,
            system_prompt: default_system_prompt(),
            debug_logging: false,
        }
    }
}

impl From<Config> for TemplateAgentConfig {
    fn from(_config: Config) -> Self {
        // Use only defaults for now, as config.agent.* does not exist
        TemplateAgentConfig::default()
    }
}
