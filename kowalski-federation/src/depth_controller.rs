use crate::FederationError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for recursive depth control
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DepthConfig {
    /// Maximum recursion depth allowed
    pub max_depth: usize,
    /// Whether to allow recursion at all
    pub allow_recursion: bool,
}

impl Default for DepthConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            allow_recursion: true,
        }
    }
}

impl DepthConfig {
    /// Creates a new depth configuration with custom max depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_depth,
            allow_recursion: true,
        }
    }

    /// Creates a non-recursive configuration
    pub fn no_recursion() -> Self {
        Self {
            max_depth: 0,
            allow_recursion: false,
        }
    }
}

/// Manages recursive depth for RLM workflows
///
/// Prevents infinite recursion by tracking the current depth level
/// and enforcing a maximum depth limit. Simplifies agent capabilities
/// at deeper levels to prevent exponential complexity growth.
///
/// # Example
///
/// ```
/// use kowalski_federation::depth_controller::{DepthController, DepthConfig};
///
/// let config = DepthConfig::with_max_depth(3);
/// let mut controller = DepthController::new(config);
///
/// // Increment depth at the start of a recursive call
/// let result = controller.increment("agent-1");
/// assert!(result.is_ok());
///
/// // Check if we can recurse further
/// if controller.can_recurse() {
///     // Perform recursive operation
/// }
///
/// // Decrement depth when done
/// let result = controller.decrement();
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthController {
    config: DepthConfig,
    current_depth: usize,
    depth_stack: Vec<String>, // Track agent IDs at each level for debugging
}

impl DepthController {
    /// Creates a new depth controller with the given configuration
    pub fn new(config: DepthConfig) -> Self {
        Self {
            config,
            current_depth: 0,
            depth_stack: Vec::new(),
        }
    }

    /// Creates a controller with default configuration (max_depth = 3)
    pub fn with_defaults() -> Self {
        Self::new(DepthConfig::default())
    }

    /// Creates a non-recursive controller
    pub fn no_recursion() -> Self {
        Self::new(DepthConfig::no_recursion())
    }

    /// Increments the recursion depth
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent entering this depth level
    ///
    /// # Returns
    /// - `Ok(())` if depth was successfully incremented
    /// - `Err(FederationError::DepthExceeded)` if max depth reached
    /// - `Err(FederationError::ProtocolViolation)` if recursion disabled
    pub fn increment(&mut self, agent_id: String) -> Result<(), FederationError> {
        if !self.config.allow_recursion && self.current_depth > 0 {
            return Err(FederationError::ProtocolViolation(
                "Recursion is disabled for this federation".to_string(),
            ));
        }

        if self.current_depth >= self.config.max_depth {
            return Err(FederationError::DepthExceeded {
                max: self.config.max_depth,
                current: self.current_depth,
            });
        }

        self.current_depth += 1;
        self.depth_stack.push(agent_id);
        Ok(())
    }

    /// Decrements the recursion depth
    ///
    /// # Returns
    /// - `Ok(())` if depth was successfully decremented
    /// - `Err(FederationError::ProtocolViolation)` if already at depth 0
    pub fn decrement(&mut self) -> Result<(), FederationError> {
        if self.current_depth == 0 {
            return Err(FederationError::ProtocolViolation(
                "Cannot decrement depth below 0".to_string(),
            ));
        }

        self.current_depth -= 1;
        self.depth_stack.pop();
        Ok(())
    }

    /// Returns the current recursion depth
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }

    /// Returns the maximum allowed depth
    pub fn max_depth(&self) -> usize {
        self.config.max_depth
    }

    /// Returns true if we're at maximum depth
    pub fn at_max(&self) -> bool {
        self.current_depth >= self.config.max_depth
    }

    /// Returns true if we can recurse further
    pub fn can_recurse(&self) -> bool {
        self.config.allow_recursion && self.current_depth < self.config.max_depth
    }

    /// Returns the number of remaining depth levels
    pub fn remaining_depth(&self) -> usize {
        if self.current_depth >= self.config.max_depth {
            0
        } else {
            self.config.max_depth - self.current_depth
        }
    }

    /// Returns a reference to the depth stack (agent IDs at each level)
    pub fn depth_stack(&self) -> &[String] {
        &self.depth_stack
    }

    /// Returns true if agent should have simplified capabilities
    ///
    /// Agents at depth 2+ should have simplified capabilities to prevent
    /// exponential complexity growth in recursive workflows.
    pub fn should_simplify_agent(&self) -> bool {
        self.current_depth >= 2
    }

    /// Resets the depth controller to initial state
    pub fn reset(&mut self) {
        self.current_depth = 0;
        self.depth_stack.clear();
    }

    /// Returns a copy of the configuration
    pub fn config(&self) -> DepthConfig {
        self.config
    }

    /// Updates the configuration
    ///
    /// Note: This resets the current depth to 0
    pub fn set_config(&mut self, config: DepthConfig) {
        self.config = config;
        self.reset();
    }
}

impl Default for DepthController {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl fmt::Display for DepthController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DepthController(current: {}/{}, stack: {:?})",
            self.current_depth, self.config.max_depth, self.depth_stack
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_controller() {
        let controller = DepthController::new(DepthConfig::with_max_depth(3));
        assert_eq!(controller.current_depth(), 0);
        assert_eq!(controller.max_depth(), 3);
        assert!(!controller.at_max());
        assert!(controller.can_recurse());
    }

    #[test]
    fn test_increment_depth() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        let result = controller.increment("agent-1".to_string());
        assert!(result.is_ok());
        assert_eq!(controller.current_depth(), 1);

        let result = controller.increment("agent-2".to_string());
        assert!(result.is_ok());
        assert_eq!(controller.current_depth(), 2);

        let result = controller.increment("agent-3".to_string());
        assert!(result.is_ok());
        assert_eq!(controller.current_depth(), 3);
        assert!(controller.at_max());
    }

    #[test]
    fn test_increment_beyond_max() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(2));

        controller.increment("agent-1".to_string()).unwrap();
        controller.increment("agent-2".to_string()).unwrap();

        let result = controller.increment("agent-3".to_string());
        assert!(result.is_err());
        match result {
            Err(FederationError::DepthExceeded { max, current }) => {
                assert_eq!(max, 2);
                assert_eq!(current, 2);
            }
            _ => panic!("Expected DepthExceeded error"),
        }
    }

    #[test]
    fn test_decrement_depth() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        controller.increment("agent-1".to_string()).unwrap();
        controller.increment("agent-2".to_string()).unwrap();
        assert_eq!(controller.current_depth(), 2);

        let result = controller.decrement();
        assert!(result.is_ok());
        assert_eq!(controller.current_depth(), 1);

        let result = controller.decrement();
        assert!(result.is_ok());
        assert_eq!(controller.current_depth(), 0);
    }

    #[test]
    fn test_decrement_below_zero() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        let result = controller.decrement();
        assert!(result.is_err());
        match result {
            Err(FederationError::ProtocolViolation(_)) => {
                // Expected
            }
            _ => panic!("Expected ProtocolViolation error"),
        }
    }

    #[test]
    fn test_depth_stack() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        controller.increment("agent-1".to_string()).unwrap();
        controller.increment("agent-2".to_string()).unwrap();
        controller.increment("agent-3".to_string()).unwrap();

        let stack = controller.depth_stack();
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0], "agent-1");
        assert_eq!(stack[1], "agent-2");
        assert_eq!(stack[2], "agent-3");
    }

    #[test]
    fn test_remaining_depth() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(5));

        assert_eq!(controller.remaining_depth(), 5);

        controller.increment("a".to_string()).unwrap();
        assert_eq!(controller.remaining_depth(), 4);

        controller.increment("b".to_string()).unwrap();
        assert_eq!(controller.remaining_depth(), 3);

        controller.decrement().unwrap();
        assert_eq!(controller.remaining_depth(), 4);
    }

    #[test]
    fn test_should_simplify_agent() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(5));

        assert!(!controller.should_simplify_agent()); // depth 0

        controller.increment("a".to_string()).unwrap();
        assert!(!controller.should_simplify_agent()); // depth 1

        controller.increment("b".to_string()).unwrap();
        assert!(controller.should_simplify_agent()); // depth 2

        controller.increment("c".to_string()).unwrap();
        assert!(controller.should_simplify_agent()); // depth 3
    }

    #[test]
    fn test_no_recursion_config() {
        let mut controller = DepthController::no_recursion();

        assert!(!controller.can_recurse());
        assert_eq!(controller.max_depth(), 0);

        let result = controller.increment("agent-1".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_reset() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        controller.increment("agent-1".to_string()).unwrap();
        controller.increment("agent-2".to_string()).unwrap();
        assert_eq!(controller.current_depth(), 2);

        controller.reset();
        assert_eq!(controller.current_depth(), 0);
        assert_eq!(controller.depth_stack().len(), 0);
    }

    #[test]
    fn test_can_recurse() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(2));

        assert!(controller.can_recurse());

        controller.increment("a".to_string()).unwrap();
        assert!(controller.can_recurse());

        controller.increment("b".to_string()).unwrap();
        assert!(!controller.can_recurse());
    }

    #[test]
    fn test_default_config() {
        let controller = DepthController::with_defaults();
        assert_eq!(controller.max_depth(), 3);
        assert!(controller.config().allow_recursion);
    }

    #[test]
    fn test_set_config() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(5));
        controller.increment("a".to_string()).unwrap();
        assert_eq!(controller.current_depth(), 1);

        let new_config = DepthConfig::with_max_depth(10);
        controller.set_config(new_config);
        assert_eq!(controller.current_depth(), 0);
        assert_eq!(controller.max_depth(), 10);
    }

    #[test]
    fn test_display() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));
        controller.increment("agent-1".to_string()).unwrap();

        let display_str = controller.to_string();
        assert!(display_str.contains("current: 1/3"));
        assert!(display_str.contains("agent-1"));
    }
}
