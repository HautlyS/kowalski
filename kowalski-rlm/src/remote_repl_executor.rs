//! Remote REPL executor via Exo cluster.

use crate::error::{RLMError, RLMResult};
use crate::exo_cluster_manager::{ExoClusterManager, REPLRequest};
use crate::repl_executor::REPLExecutor;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Remote REPL executor that routes code execution through Exo.
pub struct RemoteREPLExecutor {
    cluster: Arc<ExoClusterManager>,
    device_id: String,
    language: String,
    timeout: Duration,
    max_output_bytes: usize,
}

impl RemoteREPLExecutor {
    pub fn new(
        cluster: Arc<ExoClusterManager>,
        device_id: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        Self {
            cluster,
            device_id: device_id.into(),
            language: language.into(),
            timeout: Duration::from_secs(30),
            max_output_bytes: 1_000_000,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_output_bytes(mut self, max_output_bytes: usize) -> Self {
        self.max_output_bytes = max_output_bytes;
        self
    }
}

#[async_trait]
impl REPLExecutor for RemoteREPLExecutor {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        let request = REPLRequest {
            language: self.language.clone(),
            code: code.to_string(),
            timeout_ms: self.timeout.as_millis() as u64,
            max_output_bytes: self.max_output_bytes,
        };

        let response = self
            .cluster
            .send_repl_request(&self.device_id, request)
            .await?;

        if response.exit_code != 0 {
            return Err(RLMError::repl(format!(
                "Remote REPL failed ({}): {}",
                self.device_id, response.stderr
            )));
        }

        Ok(if response.stdout.is_empty() && response.stderr.is_empty() {
            "(no output)".to_string()
        } else if response.stdout.is_empty() {
            response.stderr
        } else {
            response.stdout
        })
    }

    fn language(&self) -> &str {
        &self.language
    }
}
