//! Device health monitoring for cross-device execution
//!
//! Tracks the health status of remote devices in an Exo cluster,
//! enabling automatic failover and device selection strategies.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Health status of a single device
#[derive(Debug, Clone)]
pub struct DeviceHealth {
    /// Unique identifier for the device
    pub device_id: String,

    /// Network address of the device
    pub address: SocketAddr,

    /// Whether the device is currently healthy and reachable
    pub is_healthy: bool,

    /// Last time health was checked
    pub last_check: Instant,

    /// Number of consecutive failures
    pub consecutive_failures: u32,

    /// Last recorded response time in milliseconds
    pub response_time_ms: u64,

    /// Device capabilities (for intelligent routing)
    pub capabilities: DeviceCapabilities,
}

/// Serializable version of DeviceHealth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDeviceHealth {
    pub device_id: String,
    pub address: SocketAddr,
    pub is_healthy: bool,
    #[serde(skip_serializing, skip_deserializing, default = "default_instant")]
    pub last_check: Instant,
    pub consecutive_failures: u32,
    pub response_time_ms: u64,
    pub capabilities: DeviceCapabilities,
}

fn default_instant() -> Instant {
    Instant::now()
}

impl From<DeviceHealth> for SerializableDeviceHealth {
    fn from(health: DeviceHealth) -> Self {
        Self {
            device_id: health.device_id,
            address: health.address,
            is_healthy: health.is_healthy,
            last_check: health.last_check,
            consecutive_failures: health.consecutive_failures,
            response_time_ms: health.response_time_ms,
            capabilities: health.capabilities,
        }
    }
}

impl From<SerializableDeviceHealth> for DeviceHealth {
    fn from(health: SerializableDeviceHealth) -> Self {
        Self {
            device_id: health.device_id,
            address: health.address,
            is_healthy: health.is_healthy,
            last_check: health.last_check,
            consecutive_failures: health.consecutive_failures,
            response_time_ms: health.response_time_ms,
            capabilities: health.capabilities,
        }
    }
}

/// Device capabilities for intelligent task routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Available programming languages/runtimes
    pub runtimes: Vec<String>, // "python", "rust", "java", "node", etc.

    /// Available GPU memory in MB
    pub gpu_memory_mb: Option<u64>,

    /// Available system memory in MB
    pub system_memory_mb: Option<u64>,

    /// Supported inference models
    pub models: Vec<String>,
}

/// Monitors health of devices in a cluster
pub struct HealthMonitor {
    devices: Arc<RwLock<Vec<DeviceHealth>>>,
    check_interval: Duration,
    /// Number of consecutive failures before marking device unhealthy
    failure_threshold: u32,
}

impl HealthMonitor {
    /// Create a new health monitor
    ///
    /// # Arguments
    /// * `check_interval` - How often to check device health
    /// * `failure_threshold` - Consecutive failures before marking unhealthy
    pub fn new(check_interval: Duration, failure_threshold: u32) -> Self {
        Self {
            devices: Arc::new(RwLock::new(Vec::new())),
            check_interval,
            failure_threshold,
        }
    }

    /// Register a new device for monitoring
    pub async fn register_device(&self, device_id: String, address: SocketAddr) {
        let mut devices = self.devices.write().await;
        
        // Avoid duplicates
        if !devices.iter().any(|d| d.device_id == device_id) {
            devices.push(DeviceHealth {
                device_id,
                address,
                is_healthy: true,
                last_check: Instant::now(),
                consecutive_failures: 0,
                response_time_ms: 0,
                capabilities: DeviceCapabilities::default(),
            });
        }
    }

    /// Register a device with capabilities
    pub async fn register_device_with_capabilities(
        &self,
        device_id: String,
        address: SocketAddr,
        capabilities: DeviceCapabilities,
    ) {
        let mut devices = self.devices.write().await;

        // Avoid duplicates
        if !devices.iter().any(|d| d.device_id == device_id) {
            devices.push(DeviceHealth {
                device_id,
                address,
                is_healthy: true,
                last_check: Instant::now(),
                consecutive_failures: 0,
                response_time_ms: 0,
                capabilities,
            });
        }
    }

    /// Check if a device is healthy
    pub async fn is_device_healthy(&self, device_id: &str) -> bool {
        let devices = self.devices.read().await;
        devices
            .iter()
            .find(|d| d.device_id == device_id)
            .map(|d| d.is_healthy)
            .unwrap_or(false)
    }

    /// Get all healthy devices
    pub async fn get_healthy_devices(&self) -> Vec<DeviceHealth> {
        let devices = self.devices.read().await;
        devices
            .iter()
            .filter(|d| d.is_healthy)
            .cloned()
            .collect()
    }

    /// Get devices that support a specific runtime
    pub async fn get_devices_with_runtime(&self, runtime: &str) -> Vec<DeviceHealth> {
        let devices = self.devices.read().await;
        devices
            .iter()
            .filter(|d| d.is_healthy && d.capabilities.runtimes.contains(&runtime.to_string()))
            .cloned()
            .collect()
    }

    /// Get the device with lowest response time for a runtime
    pub async fn get_fastest_device_for_runtime(&self, runtime: &str) -> Option<DeviceHealth> {
        let devices = self.devices.read().await;
        devices
            .iter()
            .filter(|d| d.is_healthy && d.capabilities.runtimes.contains(&runtime.to_string()))
            .min_by_key(|d| d.response_time_ms)
            .cloned()
    }

    /// Mark a device as having a failure
    pub async fn mark_failure(&self, device_id: &str) {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.iter_mut().find(|d| d.device_id == device_id) {
            device.consecutive_failures += 1;
            if device.consecutive_failures >= self.failure_threshold {
                device.is_healthy = false;
                log::warn!(
                    "Device {} marked unhealthy after {} failures",
                    device_id,
                    device.consecutive_failures
                );
            }
        }
    }

    /// Mark a device as successfully responding
    pub async fn mark_success(&self, device_id: &str, response_time_ms: u64) {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.iter_mut().find(|d| d.device_id == device_id) {
            let was_unhealthy = !device.is_healthy;
            device.consecutive_failures = 0;
            device.is_healthy = true;
            device.response_time_ms = response_time_ms;
            device.last_check = Instant::now();

            if was_unhealthy {
                log::info!("Device {} recovered and marked healthy", device_id);
            }
        }
    }

    /// Get all registered devices
    pub async fn list_all_devices(&self) -> Vec<DeviceHealth> {
        self.devices.read().await.clone()
    }

    /// Get device status summary
    pub async fn get_status(&self) -> DeviceClusterStatus {
        let devices = self.devices.read().await;
        let total = devices.len();
        let healthy = devices.iter().filter(|d| d.is_healthy).count();
        let unhealthy = total - healthy;

        DeviceClusterStatus {
            total_devices: total,
            healthy_devices: healthy,
            unhealthy_devices: unhealthy,
            average_response_time_ms: if total > 0 {
                let sum: u64 = devices.iter().map(|d| d.response_time_ms).sum();
                sum / total as u64
            } else {
                0
            },
        }
    }

    /// Start background health checks
    pub async fn start_background_checks(self: Arc<Self>) {
        let monitor = Arc::clone(&self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(monitor.check_interval).await;

                let devices: Vec<_> = {
                    let devices = monitor.devices.read().await;
                    devices
                        .iter()
                        .map(|d| (d.device_id.clone(), d.address))
                        .collect()
                };

                // Check each device
                for (device_id, address) in devices {
                    let monitor = Arc::clone(&monitor);
                    let device_id_clone = device_id.clone();
                    
                    tokio::spawn(async move {
                        // Perform actual health check
                        let start = std::time::Instant::now();
                        
                        // Try HTTP health endpoint first
                        let http_result = tokio::task::spawn_blocking(move || {
                            let url = format!("http://{}/health", address);
                            let client = reqwest::blocking::Client::builder()
                                .timeout(std::time::Duration::from_secs(5))
                                .build();
                            
                            match client {
                                Ok(client) => {
                                    match client.get(&url).send() {
                                        Ok(response) => {
                                            if response.status().is_success() {
                                                let elapsed = start.elapsed().as_millis() as u64;
                                                Some(elapsed)
                                            } else {
                                                None
                                            }
                                        }
                                        Err(_) => None,
                                    }
                                }
                                Err(_) => None,
                            }
                        }).await;
                        
                        let response_time = match http_result {
                            Ok(Some(time)) => Some(time),
                            _ => {
                                // Fallback to TCP ping if HTTP fails
                                let tcp_result = tokio::net::TcpStream::connect(address).await;
                                match tcp_result {
                                    Ok(_) => Some(start.elapsed().as_millis() as u64),
                                    Err(_) => None,
                                }
                            }
                        };
                        
                        // Update monitor based on check result
                        match response_time {
                            Some(time) => {
                                monitor.mark_success(&device_id_clone, time).await;
                                log::debug!("Health check passed for device {} ({}ms)", device_id_clone, time);
                            }
                            None => {
                                monitor.mark_failure(&device_id_clone).await;
                                log::warn!("Health check failed for device {}", device_id_clone);
                            }
                        }
                    });
                }
            }
        });
    }

    /// Remove a device from monitoring
    pub async fn unregister_device(&self, device_id: &str) {
        let mut devices = self.devices.write().await;
        devices.retain(|d| d.device_id != device_id);
    }

    /// Clear all devices
    pub async fn clear(&self) {
        self.devices.write().await.clear();
    }
}

/// Summary of cluster health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceClusterStatus {
    pub total_devices: usize,
    pub healthy_devices: usize,
    pub unhealthy_devices: usize,
    pub average_response_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_device() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);
        monitor
            .register_device("device-1".to_string(), "192.168.1.10:8080".parse().unwrap())
            .await;

        assert!(monitor.is_device_healthy("device-1").await);
    }

    #[tokio::test]
    async fn test_mark_failure_threshold() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);
        monitor
            .register_device("device-1".to_string(), "192.168.1.10:8080".parse().unwrap())
            .await;

        // Mark failures
        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;
        assert!(monitor.is_device_healthy("device-1").await);

        // Third failure should mark unhealthy
        monitor.mark_failure("device-1").await;
        assert!(!monitor.is_device_healthy("device-1").await);
    }

    #[tokio::test]
    async fn test_mark_success_recovery() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);
        monitor
            .register_device("device-1".to_string(), "192.168.1.10:8080".parse().unwrap())
            .await;

        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;
        assert!(!monitor.is_device_healthy("device-1").await);

        // Success should recover
        monitor.mark_success("device-1", 100).await;
        assert!(monitor.is_device_healthy("device-1").await);
    }

    #[tokio::test]
    async fn test_get_healthy_devices() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);

        monitor
            .register_device("device-1".to_string(), "192.168.1.10:8080".parse().unwrap())
            .await;
        monitor
            .register_device("device-2".to_string(), "192.168.1.11:8080".parse().unwrap())
            .await;

        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;

        let healthy = monitor.get_healthy_devices().await;
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].device_id, "device-2");
    }

    #[tokio::test]
    async fn test_get_devices_with_runtime() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);

        let mut caps1 = DeviceCapabilities::default();
        caps1.runtimes = vec!["python".to_string(), "node".to_string()];

        let mut caps2 = DeviceCapabilities::default();
        caps2.runtimes = vec!["rust".to_string()];

        monitor
            .register_device_with_capabilities(
                "device-1".to_string(),
                "192.168.1.10:8080".parse().unwrap(),
                caps1,
            )
            .await;

        monitor
            .register_device_with_capabilities(
                "device-2".to_string(),
                "192.168.1.11:8080".parse().unwrap(),
                caps2,
            )
            .await;

        let python_devices = monitor.get_devices_with_runtime("python").await;
        assert_eq!(python_devices.len(), 1);
        assert_eq!(python_devices[0].device_id, "device-1");
    }

    #[tokio::test]
    async fn test_cluster_status() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);

        monitor
            .register_device("device-1".to_string(), "192.168.1.10:8080".parse().unwrap())
            .await;
        monitor
            .register_device("device-2".to_string(), "192.168.1.11:8080".parse().unwrap())
            .await;

        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;
        monitor.mark_failure("device-1").await;

        let status = monitor.get_status().await;
        assert_eq!(status.total_devices, 2);
        assert_eq!(status.healthy_devices, 1);
        assert_eq!(status.unhealthy_devices, 1);
    }

    #[tokio::test]
    async fn test_unregister_device() {
        let monitor = HealthMonitor::new(Duration::from_secs(1), 3);

        monitor
            .register_device("device-1".to_string(), "192.168.1.10:8080".parse().unwrap())
            .await;

        assert!(monitor.is_device_healthy("device-1").await);

        monitor.unregister_device("device-1").await;

        assert!(!monitor.is_device_healthy("device-1").await);
    }
}
