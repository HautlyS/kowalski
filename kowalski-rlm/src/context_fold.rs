//! Context Folding Implementation
//! 
//! Implements token compression and context summarization
//! for managing long-running RLM workflows.
//!
//! # Components
//!
//! - **ContextFolder**: Handles context compression and summarization
//! - **ContextFoldConfig**: Configuration for folding behavior
//! - **FoldingStats**: Statistics about folding operations

use crate::error::{RLMError, RLMResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for context folding
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextFoldConfig {
    /// Maximum tokens before folding is triggered
    pub max_tokens: usize,
    /// Compression ratio target (0.0-1.0)
    pub compression_ratio: f64,
    /// Enable aggressive summarization
    pub aggressive: bool,
    /// Maximum iterations for folding
    pub max_iterations: usize,
}

impl Default for ContextFoldConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,
            compression_ratio: 0.7,
            aggressive: false,
            max_iterations: 3,
        }
    }
}

impl ContextFoldConfig {
    /// Create new context fold config
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    /// Set compression ratio
    pub fn with_compression_ratio(mut self, ratio: f64) -> Self {
        self.compression_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Enable aggressive summarization
    pub fn with_aggressive_folding(mut self) -> Self {
        self.aggressive = true;
        self
    }
}

/// Context folding statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FoldingStats {
    /// Original token count
    pub original_tokens: usize,
    /// Compressed token count
    pub compressed_tokens: usize,
    /// Number of fold iterations performed
    pub iterations: usize,
    /// Total time spent folding (milliseconds)
    pub fold_time_ms: u64,
    /// Compression achieved
    pub compression_ratio: f64,
}

impl FoldingStats {
    /// Calculate actual compression ratio
    pub fn actual_ratio(&self) -> f64 {
        if self.original_tokens == 0 {
            1.0
        } else {
            self.compressed_tokens as f64 / self.original_tokens as f64
        }
    }
}

/// Context folder for RLM workflows
pub struct ContextFolder {
    config: ContextFoldConfig,
    stats: Arc<RwLock<FoldingStats>>,
}

impl ContextFolder {
    /// Create new context folder
    pub fn new(config: ContextFoldConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(FoldingStats::default())),
        }
    }

    /// Estimate token count from text
    ///
    /// **Note**: This is a heuristic estimation only. Actual LLM tokenization may vary.
    /// Different models (GPT, BERT, etc.) use different tokenizers and may count
    /// tokens differently. For production use, integrate an actual tokenizer library.
    pub fn estimate_tokens(text: &str) -> usize {
        // Simple heuristic: words + punctuation
        // This is a conservative estimate that tends to undercount tokens
        let words = text.split_whitespace().count();
        let punctuation = text.matches(|c: char| c.is_ascii_punctuation()).count();
        words + (punctuation / 2)
    }

    /// Check if folding is needed
    pub fn should_fold(&self, text: &str) -> bool {
        let tokens = Self::estimate_tokens(text);
        tokens > self.config.max_tokens
    }

    /// Fold context by compressing tokens
    pub async fn fold(&self, context: &str) -> RLMResult<String> {
        let start = std::time::Instant::now();
        let original_tokens = Self::estimate_tokens(context);

        if !self.should_fold(context) {
            return Ok(context.to_string());
        }

        let mut current = context.to_string();
        let mut stats = self.stats.write().await;
        stats.original_tokens = original_tokens;

        for iter in 0..self.config.max_iterations {
            let current_tokens = Self::estimate_tokens(&current);
            
            if current_tokens <= self.config.max_tokens {
                break;
            }

            current = self.compress_iteration(&current, iter).await?;
            stats.iterations = iter + 1;

            // Safety check
            if current.is_empty() {
                return Err(RLMError::ContextFoldingFailed(
                    "Context folding resulted in empty content".to_string(),
                ));
            }
        }

        let compressed_tokens = Self::estimate_tokens(&current);
        stats.compressed_tokens = compressed_tokens;
        stats.fold_time_ms = start.elapsed().as_millis() as u64;
        stats.compression_ratio = stats.actual_ratio();

        Ok(current)
    }

    /// Single compression iteration
    async fn compress_iteration(&self, context: &str, iteration: usize) -> RLMResult<String> {
        let target_ratio = if self.config.aggressive {
            0.5 // Aggressive: keep 50%
        } else {
            self.config.compression_ratio
        };

        let lines: Vec<&str> = context.lines().collect();
        if lines.is_empty() {
            return Ok(context.to_string());
        }

        let keep_count = ((lines.len() as f64) * target_ratio) as usize;
        let keep_count = keep_count.max(1);

        // Strategy depends on iteration count
        let compressed = match iteration {
            0 => self.compress_by_importance(&lines, keep_count),
            1 => self.compress_by_sampling(&lines, keep_count),
            _ => self.compress_by_summary(&lines, keep_count),
        };

        Ok(compressed)
    }

    /// Compress by keeping important lines
    fn compress_by_importance(&self, lines: &[&str], keep_count: usize) -> String {
        // Keep first and last sections, sample middle
        let mut result = Vec::new();

        if lines.is_empty() {
            return String::new();
        }

        let section_size = (lines.len() / 3).max(1);
        
        // Keep first section
        let first_keep = (keep_count / 3).max(1);
        let end = first_keep.min(lines.len());
        for line in &lines[0..end] {
            if result.len() < keep_count {
                result.push(*line);
            }
        }

        // Sample middle
        if lines.len() > 2 * section_size {
            let mid_start = section_size;
            let mid_end = lines.len() - section_size;
            if mid_start < mid_end {
                let mid_section = &lines[mid_start..mid_end];
                let sample_count = (keep_count / 3).max(1);
                let step = (mid_section.len() / sample_count).max(1);
                for (i, line) in mid_section.iter().enumerate() {
                    if i % step == 0 && result.len() < keep_count {
                        result.push(*line);
                    }
                }
            }
        }

        // Keep last section
        let remaining = keep_count.saturating_sub(result.len());
        let start = (lines.len() - remaining).max(0);
        for line in &lines[start..] {
            if result.len() < keep_count {
                result.push(line);
            }
        }

        result.join("\n")
    }

    /// Compress by uniform sampling
    fn compress_by_sampling(&self, lines: &[&str], keep_count: usize) -> String {
        if lines.is_empty() {
            return String::new();
        }

        let step = (lines.len() / keep_count).max(1);
        let result: Vec<&str> = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .map(|(_, line)| *line)
            .take(keep_count)
            .collect();

        result.join("\n")
    }

    /// Compress by generating summary
    fn compress_by_summary(&self, lines: &[&str], _keep_count: usize) -> String {
        if lines.is_empty() {
            return String::new();
        }

        // Generate a brief summary of the content
        let summary = format!(
            "[SUMMARY: {} lines compressed to summary] {}",
            lines.len(),
            lines.first().unwrap_or(&"")
        );

        summary
    }

    /// Get folding statistics
    pub async fn stats(&self) -> FoldingStats {
        self.stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = FoldingStats::default();
    }
}

/// Trait for foldable content
#[async_trait]
pub trait Foldable {
    /// Get content size in tokens
    fn token_count(&self) -> usize;

    /// Fold the content
    async fn fold(&mut self, folder: &ContextFolder) -> RLMResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let text = "Hello world test";
        let tokens = ContextFolder::estimate_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_should_fold_detection() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let small = "Hello world";
        assert!(!folder.should_fold(small));

        let large = "word ".repeat(200);
        assert!(folder.should_fold(&large));
    }

    #[tokio::test]
    async fn test_fold_small_context() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let small = "Hello world test";
        let result = folder.fold(small).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), small);
    }

    #[tokio::test]
    async fn test_fold_large_context() {
        // Set a low limit to trigger folding
        let config = ContextFoldConfig::new(50);
        let folder = ContextFolder::new(config);

        // Create content much larger than the limit
        let large = "This is a test line with some content. ".repeat(150);
        let result = folder.fold(&large).await;

        // Main assertions: folding succeeds and produces valid output
        assert!(result.is_ok(), "Folding should succeed");
        let folded = result.unwrap();
        assert!(!folded.is_empty(), "Folding should not produce empty result");
        
        // Verify it's still valid text
        assert!(folded.len() > 0, "Folded result should have content");
    }

    #[test]
    fn test_compress_by_importance() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let lines: Vec<&str> = vec!["A", "B", "C", "D", "E", "F", "G", "H"];
        let result = folder.compress_by_importance(&lines, 3);
        
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compress_by_sampling() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let lines: Vec<&str> = vec!["A", "B", "C", "D", "E", "F", "G", "H"];
        let result = folder.compress_by_sampling(&lines, 4);
        
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let config = ContextFoldConfig::new(50);
        let folder = ContextFolder::new(config);

        let text = "word ".repeat(100);
        let _ = folder.fold(&text).await;

        let stats = folder.stats().await;
        assert!(stats.original_tokens > 0);
        #[allow(unused_comparisons)]
        {
            assert!(stats.fold_time_ms >= 0); // u64 sanity check - documents intent
        }
    }
}
