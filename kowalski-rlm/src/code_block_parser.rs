use regex::Regex;
use crate::error::{RLMError, RLMResult};
use lazy_static::lazy_static;

/// Represents a parsed code block with language and code content
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}

/// Parser for extracting code blocks from text
pub struct CodeBlockParser {
    markdown_fence_regex: Regex,
    tilde_fence_regex: Regex,
    indented_code_regex: Regex,
}

lazy_static! {
    // Matches ```language\ncode\n```
    static ref MARKDOWN_FENCE: Regex = 
        Regex::new(r"```([^\n]*)\n([\s\S]*?)```").unwrap();
    
    // Matches ~~~language\ncode\n~~~
    static ref TILDE_FENCE: Regex = 
        Regex::new(r"~~~([^\n]*)\n([\s\S]*?)~~~").unwrap();
    
    // Matches indented code blocks (4 spaces or tab)
    static ref INDENTED_CODE: Regex = 
        Regex::new(r"(?:^|\n)((?:    |\t)[^\n]*(?:\n(?:    |\t)[^\n]*)*)").unwrap();
}

impl CodeBlockParser {
    /// Create a new CodeBlockParser
    pub fn new() -> Self {
        CodeBlockParser {
            markdown_fence_regex: MARKDOWN_FENCE.clone(),
            tilde_fence_regex: TILDE_FENCE.clone(),
            indented_code_regex: INDENTED_CODE.clone(),
        }
    }

    /// Extract all code blocks from text
    ///
    /// Returns a vector of (language, code) tuples.
    /// Supports markdown fences (```), tilde fences (~~~), and indented blocks.
    pub fn extract_from(&self, text: &str) -> RLMResult<Vec<CodeBlock>> {
        let mut blocks = Vec::new();

        // Extract markdown fences first
        for caps in self.markdown_fence_regex.captures_iter(text) {
            if let (Some(lang_match), Some(code_match)) = (caps.get(1), caps.get(2)) {
                let language = lang_match.as_str().trim().to_lowercase();
                let code = code_match.as_str().to_string();

                if self.is_supported_language(&language) {
                    blocks.push(CodeBlock {
                        language: self.normalize_language(&language),
                        code: code.trim().to_string(),
                    });
                }
            }
        }

        // Extract tilde fences
        for caps in self.tilde_fence_regex.captures_iter(text) {
            if let (Some(lang_match), Some(code_match)) = (caps.get(1), caps.get(2)) {
                let language = lang_match.as_str().trim().to_lowercase();
                let code = code_match.as_str().to_string();

                if self.is_supported_language(&language) {
                    blocks.push(CodeBlock {
                        language: self.normalize_language(&language),
                        code: code.trim().to_string(),
                    });
                }
            }
        }

        // Extract indented code blocks (assume python if not specified)
        for caps in self.indented_code_regex.captures_iter(text) {
            if let Some(code_match) = caps.get(1) {
                let code = code_match
                    .as_str()
                    .lines()
                    .map(|line| {
                        if line.starts_with("    ") {
                            &line[4..]
                        } else if line.starts_with('\t') {
                            &line[1..]
                        } else {
                            line
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                blocks.push(CodeBlock {
                    language: "python".to_string(),
                    code: code.trim().to_string(),
                });
            }
        }

        Ok(blocks)
    }

    /// Detect language from code hint string
    pub fn detect_language(&self, hint: &str) -> Option<String> {
        let hint = hint.trim().to_lowercase();
        
        if self.is_supported_language(&hint) {
            Some(self.normalize_language(&hint))
        } else {
            None
        }
    }

    /// Check if language is supported
    fn is_supported_language(&self, lang: &str) -> bool {
        matches!(
            lang,
            "python"
                | "py"
                | "python3"
                | "python2"
                | "rust"
                | "rs"
                | "java"
                | "javascript"
                | "js"
                | "bash"
                | "sh"
                | "shell"
        )
    }

    /// Normalize language name to standard form
    fn normalize_language(&self, raw: &str) -> String {
        match raw.trim().to_lowercase().as_str() {
            "python" | "py" | "python3" | "python2" => "python".to_string(),
            "rust" | "rs" => "rust".to_string(),
            "java" => "java".to_string(),
            "javascript" | "js" => "javascript".to_string(),
            "bash" | "sh" | "shell" => "bash".to_string(),
            _ => raw.to_lowercase(),
        }
    }
}

impl Default for CodeBlockParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_python_markdown() {
        let parser = CodeBlockParser::new();
        let text = "Here's Python code:\n```python\nprint('hello')\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "python");
        assert_eq!(blocks[0].code, "print('hello')");
    }

    #[test]
    fn test_extract_multiple_blocks() {
        let parser = CodeBlockParser::new();
        let text = r#"
First:
```python
x = 1
```

Second:
```rust
fn main() {}
```
"#;
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, "python");
        assert_eq!(blocks[1].language, "rust");
    }

    #[test]
    fn test_extract_rust() {
        let parser = CodeBlockParser::new();
        let text = "```rust\nfn main() { println!(\"hi\"); }\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "rust");
        assert!(blocks[0].code.contains("main"));
    }

    #[test]
    fn test_extract_java() {
        let parser = CodeBlockParser::new();
        let text = "```java\npublic class Hello {}\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "java");
    }

    #[test]
    fn test_extract_javascript() {
        let parser = CodeBlockParser::new();
        let text = "```javascript\nconst x = 1;\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "javascript");
    }

    #[test]
    fn test_extract_bash() {
        let parser = CodeBlockParser::new();
        let text = "```bash\necho hello\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "bash");
    }

    #[test]
    fn test_extract_tilde_fence() {
        let parser = CodeBlockParser::new();
        let text = "~~~python\nx = 1\n~~~";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "python");
    }

    #[test]
    fn test_normalize_language() {
        let parser = CodeBlockParser::new();
        
        assert_eq!(parser.detect_language("Python"), Some("python".to_string()));
        assert_eq!(parser.detect_language("PY"), Some("python".to_string()));
        assert_eq!(parser.detect_language("Rust"), Some("rust".to_string()));
        assert_eq!(parser.detect_language("JavaScript"), Some("javascript".to_string()));
        assert_eq!(parser.detect_language("JS"), Some("javascript".to_string()));
    }

    #[test]
    fn test_unsupported_language() {
        let parser = CodeBlockParser::new();
        let text = "```c++\nint x = 1;\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        // C++ is not supported, so no blocks should be extracted
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_code_with_special_chars() {
        let parser = CodeBlockParser::new();
        let text = r#"```python
s = "This has ``` in it"
print(s)
```"#;
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].code.contains("```"));
    }

    #[test]
    fn test_empty_code_block() {
        let parser = CodeBlockParser::new();
        let text = "```python\n```";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].code.is_empty() || blocks[0].code.trim().is_empty());
    }

    #[test]
    fn test_no_code_blocks() {
        let parser = CodeBlockParser::new();
        let text = "Just plain text with no code blocks";
        let blocks = parser.extract_from(text).unwrap();
        
        assert_eq!(blocks.len(), 0);
    }
}
