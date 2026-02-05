//! Bounded conversation manager with LRU eviction
//!
//! Prevents unbounded memory growth from accumulated conversations.

use crate::conversation::Conversation;
use std::collections::HashMap;

/// Manages conversations with bounded memory (LRU eviction)
///
/// Once max_conversations is reached, the oldest conversation is removed
/// when a new one is added.
#[derive(Debug)]
pub struct ConversationManager {
    conversations: HashMap<String, Conversation>,
    insertion_order: Vec<String>, // Track insertion order for LRU eviction
    max_conversations: usize,
}

impl ConversationManager {
    /// Create a new conversation manager with specified maximum
    ///
    /// # Arguments
    /// * `max_conversations` - Maximum number of conversations to hold in memory
    ///
    /// # Example
    /// ```ignore
    /// let manager = ConversationManager::new(100); // Keep at most 100 conversations
    /// ```
    pub fn new(max_conversations: usize) -> Self {
        Self {
            conversations: HashMap::new(),
            insertion_order: Vec::new(),
            max_conversations,
        }
    }

    /// Insert a conversation, evicting oldest if at capacity
    pub fn insert(&mut self, id: String, conversation: Conversation) {
        // If at capacity, remove oldest (first inserted)
        if self.conversations.len() >= self.max_conversations {
            if let Some(evicted_id) = self.insertion_order.first().cloned() {
                self.conversations.remove(&evicted_id);
                log::debug!("Evicted conversation {} due to LRU capacity", evicted_id);
            }
        }

        // If already exists, update and move to end (most recently used)
        if self.conversations.contains_key(&id) {
            self.conversations.insert(id.clone(), conversation);
            // Update insertion order (move to end)
            if let Some(pos) = self.insertion_order.iter().position(|x| x == &id) {
                self.insertion_order.remove(pos);
                self.insertion_order.push(id);
            }
        } else {
            self.conversations.insert(id.clone(), conversation);
            self.insertion_order.push(id);
        }
    }

    /// Get a conversation by ID
    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.conversations.get(id)
    }

    /// Get a mutable reference to a conversation by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Conversation> {
        // Move to end (most recently used) when accessed
        if self.conversations.contains_key(id) {
            if let Some(pos) = self.insertion_order.iter().position(|x| x == id) {
                let id_str = self.insertion_order.remove(pos);
                self.insertion_order.push(id_str);
            }
        }
        self.conversations.get_mut(id)
    }

    /// Remove a conversation and return it
    pub fn remove(&mut self, id: &str) -> Option<Conversation> {
        let result = self.conversations.remove(id);
        if let Some(pos) = self.insertion_order.iter().position(|x| x == id) {
            self.insertion_order.remove(pos);
        }
        result
    }

    /// List all conversation IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.insertion_order.clone()
    }

    /// Get all conversations (references) in LRU order
    pub fn list_all(&self) -> Vec<&Conversation> {
        self.insertion_order
            .iter()
            .filter_map(|id| self.conversations.get(id))
            .collect()
    }

    /// Check if a conversation exists
    pub fn contains(&self, id: &str) -> bool {
        self.conversations.contains_key(id)
    }

    /// Get the current number of conversations
    pub fn len(&self) -> usize {
        self.conversations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.conversations.is_empty()
    }

    /// Get the maximum capacity
    pub fn capacity(&self) -> usize {
        self.max_conversations
    }

    /// Clear all conversations
    pub fn clear(&mut self) {
        self.conversations.clear();
        self.insertion_order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_lru_eviction() {
        let mut manager = ConversationManager::new(2);

        let mut conv1 = Conversation::new("model");
        conv1.add_message("user", "Hello");
        manager.insert("conv1".to_string(), conv1);

        let mut conv2 = Conversation::new("model");
        conv2.add_message("user", "Hi");
        manager.insert("conv2".to_string(), conv2);

        assert_eq!(manager.len(), 2);
        assert!(manager.contains("conv1"));
        assert!(manager.contains("conv2"));

        // Third conversation should evict first
        let mut conv3 = Conversation::new("model");
        conv3.add_message("user", "Hey");
        manager.insert("conv3".to_string(), conv3);

        assert_eq!(manager.len(), 2);
        assert!(!manager.contains("conv1")); // Evicted
        assert!(manager.contains("conv2"));
        assert!(manager.contains("conv3"));
    }

    #[test]
    fn test_insert_and_get() {
        let mut manager = ConversationManager::new(10);
        let conversation = Conversation::new("model");
        conversation.add_message("user", "test");

        manager.insert("id1".to_string(), conversation.clone());

        assert!(manager.contains("id1"));
        let retrieved = manager.get("id1").unwrap();
        assert_eq!(retrieved.id, "id1");
        assert_eq!(retrieved.model, "model");
    }

    #[test]
    fn test_get_mut_updates_lru() {
        let mut manager = ConversationManager::new(3);

        let mut conv1 = Conversation::new("model");
        conv1.add_message("user", "First");
        manager.insert("id1".to_string(), conv1);

        let mut conv2 = Conversation::new("model");
        conv2.add_message("user", "Second");
        manager.insert("id2".to_string(), conv2);

        let mut conv3 = Conversation::new("model");
        conv3.add_message("user", "Third");
        manager.insert("id3".to_string(), conv3);

        // Access conv1 (should move to end)
        if let Some(conv) = manager.get_mut("id1") {
            conv.add_message("user", "Updated");
        }

        let ids = manager.list_ids();
        assert_eq!(
            ids,
            vec!["id2".to_string(), "id3".to_string(), "id1".to_string()]
        );
    }

    #[test]
    fn test_remove() {
        let mut manager = ConversationManager::new(10);
        let conversation = Conversation::new("model");
        manager.insert("id1".to_string(), conversation);

        assert!(manager.contains("id1"));
        manager.remove("id1");
        assert!(!manager.contains("id1"));
    }

    #[test]
    fn test_list_ids() {
        let mut manager = ConversationManager::new(10);

        manager.insert("id1".to_string(), Conversation::new("model"));
        manager.insert("id2".to_string(), Conversation::new("model"));
        manager.insert("id3".to_string(), Conversation::new("model"));

        let ids = manager.list_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"id1".to_string()));
    }

    #[test]
    fn test_capacity_limit() {
        let manager = ConversationManager::new(5);
        assert_eq!(manager.capacity(), 5);
    }

    #[test]
    fn test_is_empty() {
        let mut manager = ConversationManager::new(10);
        assert!(manager.is_empty());

        manager.insert("id1".to_string(), Conversation::new("model"));
        assert!(!manager.is_empty());

        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_list_all_preserves_order() {
        let mut manager = ConversationManager::new(10);

        manager.insert("id1".to_string(), Conversation::new("model"));
        manager.insert("id2".to_string(), Conversation::new("model"));
        manager.insert("id3".to_string(), Conversation::new("model"));

        let all = manager.list_all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, "id1");
        assert_eq!(all[1].id, "id2");
        assert_eq!(all[2].id, "id3");
    }
}
