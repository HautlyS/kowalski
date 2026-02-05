use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;
use tracing::{info, debug};

use crate::{FederationError, FederationMessage, message::MessageType};
use kowalski_core::agent::MessageHandler;
use kowalski_core::{Agent, BaseAgent};

/// Represents the role of an agent in the federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FederationRole {
    /// Coordinator manages the federation and orchestrates tasks
    Coordinator,
    /// Worker performs tasks assigned by the coordinator
    Worker,
    /// Observer watches federation activities without participating
    Observer,
}

/// Trait for agents that can participate in a federation
#[async_trait]
pub trait FederatedAgent: Agent {
    /// Get the agent's unique identifier within the federation
    fn federation_id(&self) -> &str;

    /// Get the agent's role in the federation
    fn federation_role(&self) -> FederationRole;

    /// Set the agent's role in the federation
    fn set_federation_role(&mut self, role: FederationRole);

    /// Register with the federation coordinator
    async fn register_with_coordinator(&mut self, coordinator: &str)
    -> Result<(), FederationError>;

    /// Send a message to another federated agent
    async fn send_message(
        &self,
        recipient: &str,
        message: FederationMessage,
    ) -> Result<(), FederationError>;

    /// Broadcast a message to all agents in the federation
    async fn broadcast_message(&self, message: FederationMessage) -> Result<(), FederationError>;

    /// Handle incoming federation message
    async fn handle_federation_message(
        &mut self,
        message: FederationMessage,
    ) -> Result<(), FederationError>;
}

/// Federated agent with registry reference for communication
pub struct FederatedAgentState {
    /// Registry reference for message passing
    registry: Option<Arc<crate::registry::AgentRegistry>>,
    /// Coordinator identifier
    coordinator_id: Option<String>,
    /// Incoming messages queue
    inbox: Arc<RwLock<Vec<FederationMessage>>>,
}

impl Default for FederatedAgentState {
    fn default() -> Self {
        Self {
            registry: None,
            coordinator_id: None,
            inbox: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// Thread-safe federated state storage
static FEDERATED_STATE: LazyLock<std::sync::RwLock<std::collections::HashMap<String, FederatedAgentState>>> = 
    LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

/// Implementation of FederatedAgent for BaseAgent
#[async_trait]
impl FederatedAgent for BaseAgent {
    fn federation_id(&self) -> &str {
        &self.name
    }

    fn federation_role(&self) -> FederationRole {
        FederationRole::Worker
    }

    fn set_federation_role(&mut self, role: FederationRole) {
        info!("Setting federation role for agent {} to: {:?}", self.name, role);
        let mut state = FEDERATED_STATE.write().unwrap();
        state.entry(self.name.clone()).or_default();
    }

    async fn register_with_coordinator(
        &mut self,
        coordinator: &str,
    ) -> Result<(), FederationError> {
        let agent_id = self.federation_id().to_string();
        
        {
            let mut state = FEDERATED_STATE.write().unwrap();
            let state_entry = state.entry(agent_id.clone()).or_default();
            state_entry.coordinator_id = Some(coordinator.to_string());
        }
        
        info!("Agent {} registered with coordinator: {}", agent_id, coordinator);
        
        Ok(())
    }

    async fn send_message(
        &self,
        recipient: &str,
        message: FederationMessage,
    ) -> Result<(), FederationError> {
        debug!("Agent {} sending message to {}: {:?}", self.federation_id(), recipient, message.message_type);
        
        // In a real implementation, this would:
        // 1. Serialize the message
        // 2. Send it over the network (HTTP, WebSocket, gRPC, etc.)
        // 3. Handle network errors and retries
        
        // For now, we simulate message delivery
        let json_bytes = serde_json::to_vec(&message)
            .map_err(|e| FederationError::SerializationError(e.to_string()))?;
        
        // Log the message size for demonstration
        debug!("Message serialized to {} bytes", json_bytes.len());
        
        // In a production implementation, we would send this over the network
        // Example: self.network_client.send(recipient, json_bytes).await?;
        
        Ok(())
    }

    async fn broadcast_message(&self, message: FederationMessage) -> Result<(), FederationError> {
        debug!("Agent {} broadcasting message: {:?}", self.federation_id(), message.message_type);
        
        // In a real implementation, this would:
        // 1. Get list of all registered agents from registry
        // 2. Send message to each agent except sender
        // 3. Handle partial failures
        
        // For now, we simulate broadcast
        let json_bytes = serde_json::to_vec(&message)
            .map_err(|e| FederationError::SerializationError(e.to_string()))?;
        
        debug!("Broadcast message serialized to {} bytes", json_bytes.len());
        
        // In a production implementation:
        // let recipients = self.registry.list_agents().await;
        // for (recipient_id, _) in recipients {
        //     if recipient_id != self.federation_id() {
        //         self.send_message(&recipient_id, message.clone()).await?;
        //     }
        // }
        
        Ok(())
    }

    async fn handle_federation_message(
        &mut self,
        message: FederationMessage,
    ) -> Result<(), FederationError> {
        debug!("Agent {} received message: {:?}", self.federation_id(), message.message_type);
        
        match message.message_type {
            MessageType::Register => {
                info!("Received registration message from: {}", message.sender);
            }
            MessageType::TaskDelegation => {
                info!("Received task delegation from: {}", message.sender);
                // Parse task from message content
                if let Ok(task) = serde_json::from_str::<crate::orchestrator::FederationTask>(&message.content) {
                    debug!("Task ID: {}, Type: {}", task.id, task.task_type);
                    // Process task logic would go here
                }
            }
            MessageType::TaskCompletion => {
                info!("Received task completion from: {}", message.sender);
            }
            MessageType::Status => {
                debug!("Received status update from: {}", message.sender);
            }
            MessageType::Error => {
                debug!("Received error from: {}: {}", message.sender, message.content);
            }
            MessageType::Custom(ref custom_type) => {
                debug!("Received custom message type: {}", custom_type);
            }
        }
        
        // Store message in inbox for later processing
        let agent_id = self.federation_id().to_string();
        let inbox_ptr = {
            let state = FEDERATED_STATE.read().unwrap();
            state.get(&agent_id).map(|s| Arc::clone(&s.inbox))
        };
        
        if let Some(inbox) = inbox_ptr {
            let mut inbox_write = inbox.write().await;
            inbox_write.push(message);
        }
        
        Ok(())
    }
}

pub struct FederatedBaseAgent(pub BaseAgent);

#[async_trait::async_trait]
impl MessageHandler for FederatedBaseAgent {
    type Message = FederationMessage;
    type Error = FederationError;

    async fn handle_message(&mut self, message: Self::Message) -> Result<(), Self::Error> {
        debug!("FederatedBaseAgent handling message: {:?}", message.message_type);
        
        // Delegate to inner BaseAgent's federation message handler
        self.0.handle_federation_message(message).await
    }
}
