use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dynamic prompt augmentation system for RLM environments
///
/// `EnvironmentTips` provides contextual suggestions to the RLM based on
/// the current execution environment. This includes information about
/// available tools, resource limits, execution context, and optimization hints.
///
/// # Example
///
/// ```no_run
/// use kowalski_core::rlm::EnvironmentTips;
///
/// let tips = EnvironmentTips::new()
///     .add_tip("web_search", "Use for recent information")
///     .add_tip("code_execution", "Python 3.9+ available")
///     .add_resource("max_iterations", "5")
///     .add_resource("timeout_seconds", "300");
///
/// let prompt = tips.augment_prompt("Find the latest AI papers");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentTips {
    /// Tool-specific tips (name -> suggestion)
    tips: HashMap<String, String>,
    /// Resource constraints and capabilities
    resources: HashMap<String, String>,
    /// Execution context information
    context: HashMap<String, String>,
}

impl EnvironmentTips {
    /// Creates a new, empty environment tips system
    pub fn new() -> Self {
        Self {
            tips: HashMap::new(),
            resources: HashMap::new(),
            context: HashMap::new(),
        }
    }

    /// Adds a tool tip
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool
    /// * `suggestion` - Optimization or usage suggestion for this tool
    pub fn add_tip(mut self, tool_name: &str, suggestion: &str) -> Self {
        self.tips.insert(tool_name.to_string(), suggestion.to_string());
        self
    }

    /// Adds a resource constraint or capability
    ///
    /// # Arguments
    /// * `resource` - Name of the resource (e.g., "max_iterations", "timeout_seconds")
    /// * `value` - Value or description of the resource
    pub fn add_resource(mut self, resource: &str, value: &str) -> Self {
        self.resources.insert(resource.to_string(), value.to_string());
        self
    }

    /// Adds execution context information
    ///
    /// # Arguments
    /// * `key` - Context key
    /// * `value` - Context value
    pub fn add_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Augments a prompt with environment tips
    ///
    /// Returns an enhanced prompt that includes relevant environment information.
    /// This prompt augmentation helps the LLM make better decisions about
    /// tool selection, resource usage, and refinement strategy.
    ///
    /// # Arguments
    /// * `prompt` - The original user prompt
    ///
    /// # Returns
    /// The augmented prompt with environment context
    pub fn augment_prompt(&self, prompt: &str) -> String {
        let mut augmented = String::new();

        // Add the original prompt
        augmented.push_str(prompt);
        augmented.push_str("\n\n");

        // Add resource constraints
        if !self.resources.is_empty() {
            augmented.push_str("## Resource Constraints\n");
            for (resource, value) in &self.resources {
                augmented.push_str(&format!("- {}: {}\n", resource, value));
            }
            augmented.push('\n');
        }

        // Add available tools and tips
        if !self.tips.is_empty() {
            augmented.push_str("## Available Tools & Optimization Tips\n");
            for (tool, tip) in &self.tips {
                augmented.push_str(&format!("- **{}**: {}\n", tool, tip));
            }
            augmented.push('\n');
        }

        // Add execution context
        if !self.context.is_empty() {
            augmented.push_str("## Execution Context\n");
            for (key, value) in &self.context {
                augmented.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        augmented
    }

    /// Gets a specific tip for a tool
    pub fn get_tip(&self, tool_name: &str) -> Option<&str> {
        self.tips.get(tool_name).map(|s| s.as_str())
    }

    /// Gets a specific resource value
    pub fn get_resource(&self, resource: &str) -> Option<&str> {
        self.resources.get(resource).map(|s| s.as_str())
    }

    /// Gets a specific context value
    pub fn get_context(&self, key: &str) -> Option<&str> {
        self.context.get(key).map(|s| s.as_str())
    }

    /// Returns all tips as a HashMap
    pub fn tips(&self) -> &HashMap<String, String> {
        &self.tips
    }

    /// Returns all resources as a HashMap
    pub fn resources(&self) -> &HashMap<String, String> {
        &self.resources
    }

    /// Returns all context as a HashMap
    pub fn context(&self) -> &HashMap<String, String> {
        &self.context
    }
}

impl Default for EnvironmentTips {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_add_tips() {
        let tips = EnvironmentTips::new()
            .add_tip("web_search", "Use for recent info")
            .add_tip("csv_analysis", "Optimize for large datasets");

        assert_eq!(tips.get_tip("web_search"), Some("Use for recent info"));
        assert_eq!(tips.get_tip("csv_analysis"), Some("Optimize for large datasets"));
        assert_eq!(tips.get_tip("unknown"), None);
    }

    #[test]
    fn test_add_resources() {
        let tips = EnvironmentTips::new()
            .add_resource("max_iterations", "5")
            .add_resource("timeout_seconds", "300");

        assert_eq!(tips.get_resource("max_iterations"), Some("5"));
        assert_eq!(tips.get_resource("timeout_seconds"), Some("300"));
    }

    #[test]
    fn test_add_context() {
        let tips = EnvironmentTips::new()
            .add_context("user_id", "user123")
            .add_context("task_type", "research");

        assert_eq!(tips.get_context("user_id"), Some("user123"));
        assert_eq!(tips.get_context("task_type"), Some("research"));
    }

    #[test]
    fn test_augment_prompt_with_tips() {
        let tips = EnvironmentTips::new()
            .add_tip("web_search", "Use for recent information")
            .add_resource("max_iterations", "3")
            .add_context("task", "research");

        let prompt = "Find AI papers";
        let augmented = tips.augment_prompt(prompt);

        assert!(augmented.contains("Find AI papers"));
        assert!(augmented.contains("max_iterations"));
        assert!(augmented.contains("web_search"));
        assert!(augmented.contains("task"));
    }

    #[test]
    fn test_augment_prompt_without_extras() {
        let tips = EnvironmentTips::new();
        let prompt = "Simple prompt";
        let augmented = tips.augment_prompt(prompt);

        assert_eq!(augmented, prompt);
    }

    #[test]
    fn test_default_instance() {
        let tips = EnvironmentTips::default();
        assert!(tips.tips().is_empty());
        assert!(tips.resources().is_empty());
        assert!(tips.context().is_empty());
    }

    #[test]
    fn test_serialize_deserialize() {
        let original = EnvironmentTips::new()
            .add_tip("tool1", "tip1")
            .add_resource("res1", "val1")
            .add_context("ctx1", "ctxval1");

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EnvironmentTips = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.get_tip("tool1"), original.get_tip("tool1"));
        assert_eq!(deserialized.get_resource("res1"), original.get_resource("res1"));
        assert_eq!(deserialized.get_context("ctx1"), original.get_context("ctx1"));
    }
}
