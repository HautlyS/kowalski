use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Stores the iteratively refined answer from RLM execution
///
/// The `AnswerBuffer` accumulates content across multiple refinement iterations,
/// maintaining a `ready` flag that signals when the final answer is complete.
/// This is critical for RLM's iterative refinement pattern where multiple
/// sub-LLM calls contribute to a single, continuously-refined answer.
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use kowalski_core::rlm::AnswerBuffer;
///
/// #[tokio::main]
/// async fn example() {
///     let buffer = Arc::new(AnswerBuffer::new());
///     
///     // Append result from first refinement iteration
///     buffer.append("Initial research: ").await;
///     buffer.append("Found three relevant papers").await;
///     
///     // Finalize when done
///     buffer.finalize().await;
///     
///     // Retrieve final answer
///     let answer = buffer.get_content().await;
///     assert!(buffer.is_ready().await);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AnswerBuffer {
    inner: Arc<RwLock<AnswerBufferInner>>,
}

#[derive(Debug)]
struct AnswerBufferInner {
    content: String,
    ready: bool,
    iteration_count: usize,
}

impl AnswerBuffer {
    /// Creates a new, empty answer buffer
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AnswerBufferInner {
                content: String::new(),
                ready: false,
                iteration_count: 0,
            })),
        }
    }

    /// Appends text to the answer buffer
    ///
    /// This is called multiple times during RLM execution as each refinement
    /// iteration produces new content. Content is accumulated in order.
    ///
    /// # Arguments
    /// * `text` - The text to append to the buffer
    ///
    /// # Panics
    /// Panics if the buffer is already marked as ready (finalized)
    pub async fn append(&self, text: &str) {
        let mut inner = self.inner.write().await;
        if inner.ready {
            panic!("Cannot append to finalized answer buffer");
        }
        inner.content.push_str(text);
    }

    /// Marks the answer as complete (ready for consumption)
    ///
    /// After finalization, no more content can be appended. This signals to
    /// waiters that the answer is complete and ready for retrieval.
    pub async fn finalize(&self) {
        let mut inner = self.inner.write().await;
        inner.ready = true;
    }

    /// Waits until the answer is ready or timeout expires
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for the answer to be ready
    ///
    /// # Returns
    /// * `Ok(())` if the answer became ready
    /// * `Err(String)` if timeout occurred before ready
    pub async fn wait_ready(&self, timeout: Duration) -> Result<(), String> {
        let start = std::time::Instant::now();
        loop {
            {
                let inner = self.inner.read().await;
                if inner.ready {
                    return Ok(());
                }
            }
            if start.elapsed() > timeout {
                return Err("Answer buffer timeout".to_string());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Returns the current content of the answer buffer
    ///
    /// May be called before `finalize()` to get partial results, or
    /// after `wait_ready()` to get the final answer.
    pub async fn get_content(&self) -> String {
        let inner = self.inner.read().await;
        inner.content.clone()
    }

    /// Returns whether the answer buffer is finalized and ready
    pub async fn is_ready(&self) -> bool {
        let inner = self.inner.read().await;
        inner.ready
    }

    /// Returns the number of refinement iterations completed
    pub async fn iteration_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.iteration_count
    }

    /// Increments the iteration counter (called when each refinement completes)
    pub async fn next_iteration(&self) {
        let mut inner = self.inner.write().await;
        inner.iteration_count += 1;
    }

    /// Clears the buffer and resets the ready flag
    ///
    /// Used to reset the buffer for a new RLM execution.
    pub async fn reset(&self) {
        let mut inner = self.inner.write().await;
        inner.content.clear();
        inner.ready = false;
        inner.iteration_count = 0;
    }
}

impl Default for AnswerBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_append_and_get() {
        let buffer = AnswerBuffer::new();
        buffer.append("Hello").await;
        buffer.append(" ").await;
        buffer.append("World").await;

        let content = buffer.get_content().await;
        assert_eq!(content, "Hello World");
    }

    #[tokio::test]
    async fn test_finalize_and_ready() {
        let buffer = AnswerBuffer::new();
        assert!(!buffer.is_ready().await);

        buffer.finalize().await;
        assert!(buffer.is_ready().await);
    }

    #[tokio::test]
    async fn test_wait_ready_timeout() {
        let buffer = AnswerBuffer::new();
        let result = buffer
            .wait_ready(Duration::from_millis(50))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wait_ready_success() {
        let buffer = Arc::new(AnswerBuffer::new());
        let buffer_clone = buffer.clone();

        // Spawn task to finalize after a delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            buffer_clone.finalize().await;
        });

        let result = buffer.wait_ready(Duration::from_secs(5)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_iteration_count() {
        let buffer = AnswerBuffer::new();
        assert_eq!(buffer.iteration_count().await, 0);

        buffer.next_iteration().await;
        assert_eq!(buffer.iteration_count().await, 1);

        buffer.next_iteration().await;
        assert_eq!(buffer.iteration_count().await, 2);
    }

    #[tokio::test]
    async fn test_reset() {
        let buffer = AnswerBuffer::new();
        buffer.append("Content").await;
        buffer.finalize().await;
        buffer.next_iteration().await;

        buffer.reset().await;

        assert_eq!(buffer.get_content().await, "");
        assert!(!buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 0);
    }

    #[tokio::test]
    #[should_panic(expected = "Cannot append to finalized")]
    async fn test_append_after_finalize() {
        let buffer = AnswerBuffer::new();
        buffer.finalize().await;
        buffer.append("Should panic").await;
    }
}
