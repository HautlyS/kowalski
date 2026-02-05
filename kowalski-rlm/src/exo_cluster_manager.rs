//! Exo cluster manager: minimal HTTP client integration
//!
//! Provides device discovery and remote execution APIs over Exo's HTTP interface.

use crate::device_health::{DeviceCapabilities, DeviceHealth};
use crate::error::{RLMError, RLMResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExoDeviceInfo {
    pub id: String,
    pub address: String,
    #[serde(default)]
    pub capabilities: DeviceCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExoClusterState {
    pub devices: Vec<ExoDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExoModelInfo {
    pub name: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExoModelListResponse {
    pub models: Vec<ExoModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct REPLRequest {
    pub language: String,
    pub code: String,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct REPLResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Manages communication with Exo cluster
#[derive(Debug)]
pub struct ExoClusterManager {
    base_url: String,
    client: reqwest::Client,
    devices: Arc<RwLock<HashMap<String, ExoDeviceInfo>>>,
}

impl ExoClusterManager {
    pub async fn new(base_url: impl Into<String>) -> RLMResult<Self> {
        let base_url = base_url.into();
        let client = reqwest::ClientBuilder::new()
            .pool_max_idle_per_host(10)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| RLMError::network(e.to_string()))?;

        let manager = Self {
            base_url,
            client,
            devices: Arc::new(RwLock::new(HashMap::new())),
        };

        manager.discover_devices().await?;
        Ok(manager)
    }

    pub async fn discover_devices(&self) -> RLMResult<()> {
        let url = format!("{}/state", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RLMError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RLMError::network(format!(
                "Exo discovery failed: {}",
                error_text
            )));
        }

        let state: ExoClusterState = response
            .json()
            .await
            .map_err(|e| RLMError::serialization(e.to_string()))?;

        let mut devices = self.devices.write().await;
        devices.clear();
        for device in state.devices {
            devices.insert(device.id.clone(), device);
        }

        Ok(())
    }

    pub async fn list_devices(&self) -> RLMResult<Vec<ExoDeviceInfo>> {
        let devices = self.devices.read().await;
        Ok(devices.values().cloned().collect())
    }

    pub async fn list_models(&self) -> RLMResult<Vec<ExoModelInfo>> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RLMError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RLMError::network(format!(
                "Exo model list failed: {}",
                error_text
            )));
        }

        let models: ExoModelListResponse = response
            .json()
            .await
            .map_err(|e| RLMError::serialization(e.to_string()))?;
        Ok(models.models)
    }

    pub async fn send_repl_request(
        &self,
        device_id: &str,
        request: REPLRequest,
    ) -> RLMResult<REPLResponse> {
        let url = format!("{}/api/repl/execute", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "device_id": device_id,
                "request": request,
            }))
            .send()
            .await
            .map_err(|e| RLMError::network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(RLMError::network(format!(
                "Exo REPL request failed: {}",
                error_text
            )));
        }

        let repl_response: REPLResponse = response
            .json()
            .await
            .map_err(|e| RLMError::serialization(e.to_string()))?;
        Ok(repl_response)
    }

    pub async fn to_device_health_snapshot(&self) -> Vec<DeviceHealth> {
        let devices = self.devices.read().await;
        devices
            .values()
            .filter_map(|device| {
                let address = device.address.parse().ok()?;
                Some(DeviceHealth {
                    device_id: device.id.clone(),
                    address,
                    is_healthy: true,
                    last_check: std::time::Instant::now(),
                    consecutive_failures: 0,
                    response_time_ms: 0,
                    capabilities: device.capabilities.clone(),
                })
            })
            .collect()
    }
}
