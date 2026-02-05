use kowalski_core::config::{Config as CoreConfig, ConfigExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAgentConfig {
    /// Core configuration
    core: CoreConfig,
    pub system_prompt: String,
    pub max_rows: usize,
    pub max_columns: usize,
}

impl ConfigExt for DataAgentConfig {
    fn core(&self) -> &CoreConfig {
        &self.core
    }

    fn core_mut(&mut self) -> &mut CoreConfig {
        &mut self.core
    }
}

impl Default for DataAgentConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            system_prompt: "You are a data analysis assistant.".to_string(),
            max_rows: 1000,
            max_columns: 50,
        }
    }
}

impl From<CoreConfig> for DataAgentConfig {
    fn from(core: CoreConfig) -> Self {
        Self {
            core,
            ..Default::default()
        }
    }
}
