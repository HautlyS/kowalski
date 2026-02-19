use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: Option<String>,
    pub default_model: String,
    pub base_url: String,
    pub site_url: Option<String>,
    pub site_name: Option<String>,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            default_model: "anthropic/claude-sonnet-4".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            site_url: Some("https://github.com/yarenty/kowalski".to_string()),
            site_name: Some("Kowalski".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    Ollama,
    OpenRouter,
    Exo,
}

impl Default for Provider {
    fn default() -> Self {
        Self::Ollama
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Ollama => write!(f, "ollama"),
            Provider::OpenRouter => write!(f, "openrouter"),
            Provider::Exo => write!(f, "exo"),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Provider::Ollama),
            "openrouter" => Ok(Provider::OpenRouter),
            "exo" => Ok(Provider::Exo),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: Provider,
    pub ollama: OllamaConfig,
    pub openrouter: OpenRouterConfig,
    pub exo: ExoConfig,
    pub chat: ChatConfig,
    pub working_memory_retrieval_limit: usize,
    pub episodic_memory_retrieval_limit: usize,
    pub semantic_memory_retrieval_limit: usize,
    pub additional: HashMap<String, serde_json::Value>,
}

/// Configuration for Ollama integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// The host where Ollama is running
    pub host: String,
    /// The port where Ollama is running
    pub port: u16,
    /// The model to use
    pub model: String,
    /// Additional Ollama-specific settings
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
            model: "llama3.2".to_string(), //llama3.2 //deepseek-r1:1.5b
            additional: HashMap::new(),
        }
    }
}

/// Configuration for chat functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Maximum number of messages to keep in history
    pub max_history: usize,
    /// Whether to enable streaming responses
    pub enable_streaming: bool,
    /// Temperature for response generation (0.0 to 1.0)
    pub temperature: f32,
    /// Maximum number of tokens in generated responses
    pub max_tokens: u32,
    /// Additional chat-specific settings
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Configuration for Exo integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExoConfig {
    /// Base URL for Exo API
    pub base_url: String,
    /// Whether to enable Exo cluster integration
    pub enable_cluster: bool,
    /// Additional Exo-specific settings
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl Default for ExoConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:52415".to_string(),
            enable_cluster: false,
            additional: HashMap::new(),
        }
    }
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            enable_streaming: true,
            temperature: 0.7,
            max_tokens: 2048,
            additional: HashMap::new(),
        }
    }
}

/// Trait for extending configuration with additional settings
pub trait ConfigExt {
    /// Get a reference to the core configuration
    fn core(&self) -> &Config;

    /// Get a mutable reference to the core configuration
    fn core_mut(&mut self) -> &mut Config;

    /// Get additional configuration value by key
    fn get_additional<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.core()
            .additional
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set additional configuration value
    fn set_additional<T: serde::Serialize>(&mut self, key: &str, value: T) {
        if let Ok(json) = serde_json::to_value(value) {
            self.core_mut().additional.insert(key.to_string(), json);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: Provider::Ollama,
            ollama: OllamaConfig::default(),
            openrouter: OpenRouterConfig::default(),
            exo: ExoConfig::default(),
            chat: ChatConfig::default(),
            working_memory_retrieval_limit: 3,
            episodic_memory_retrieval_limit: 3,
            semantic_memory_retrieval_limit: 3,
            additional: HashMap::new(),
        }
    }
}

impl Config {
    pub fn effective_model(&self) -> String {
        match self.provider {
            Provider::Ollama => self.ollama.model.clone(),
            Provider::OpenRouter => self.openrouter.default_model.clone(),
            Provider::Exo => self.ollama.model.clone(),
        }
    }

    pub fn with_openrouter(api_key: String, model: Option<String>) -> Self {
        let mut config = Self::default();
        config.provider = Provider::OpenRouter;
        config.openrouter.api_key = Some(api_key);
        if let Some(m) = model {
            config.openrouter.default_model = m;
        }
        config
    }

    pub fn apply_env_overrides(&mut self) {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if !key.is_empty() {
                self.openrouter.api_key = Some(key);
            }
        }
        if let Ok(provider) = std::env::var("KOWALSKI_PROVIDER") {
            if let Ok(p) = provider.parse::<Provider>() {
                self.provider = p;
            }
        }
        if let Ok(model) = std::env::var("KOWALSKI_MODEL") {
            if !model.is_empty() {
                match self.provider {
                    Provider::Ollama => self.ollama.model = model,
                    Provider::OpenRouter => self.openrouter.default_model = model,
                    Provider::Exo => self.ollama.model = model,
                }
            }
        }
        if let Ok(model) = std::env::var("OLLAMA_MODEL") {
            if !model.is_empty() {
                self.ollama.model = model;
            }
        }
    }
}
