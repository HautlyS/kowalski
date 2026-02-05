use async_trait::async_trait;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use crate::error::{RLMError, RLMResult};
use uuid::Uuid;

/// Trait for REPL executors
#[async_trait]
pub trait REPLExecutor: Send + Sync {
    /// Execute code and return output
    async fn execute(&self, code: &str) -> RLMResult<String>;
    
    /// Get the language this executor handles
    fn language(&self) -> &str;
}

/// Python REPL Executor
pub struct PythonREPL {
    timeout: Duration,
}

/// Rust REPL Executor
pub struct RustREPL {
    timeout: Duration,
}

/// Java REPL Executor
pub struct JavaREPL {
    timeout: Duration,
}

/// Bash/Shell REPL Executor
pub struct BashREPL {
    timeout: Duration,
}

/// JavaScript REPL Executor
pub struct JavaScriptREPL {
    timeout: Duration,
}

impl PythonREPL {
    pub fn new() -> Self {
        PythonREPL {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for PythonREPL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl REPLExecutor for PythonREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        // Create temp directory that auto-cleans on drop
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp dir: {}", e)))?;
        
        let temp_file = temp_dir.path().join(format!("{}.py", Uuid::new_v4()));

        let mut file = fs::File::create(&temp_file)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp file: {}", e)))?;

        file.write_all(code.as_bytes())
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write code: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to sync file: {}", e)))?;

        drop(file);

        let child = Command::new("python3")
            .arg(&temp_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to spawn Python: {}", e)))?;

        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(result) => result.map_err(|e| {
                RLMError::ExecutionError(format!("Failed to wait for Python: {}", e))
            })?,
            Err(_) => {
                return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() {
            return Err(RLMError::REPLError(format!(
                "Python execution failed:\n{}",
                stderr
            )));
        }

        Ok(if stdout.is_empty() && stderr.is_empty() {
            "(no output)".to_string()
        } else if stdout.is_empty() {
            stderr
        } else {
            stdout
        })
    }

    fn language(&self) -> &str {
        "python"
    }
}

impl RustREPL {
    pub fn new() -> Self {
        RustREPL {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for RustREPL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl REPLExecutor for RustREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp dir: {}", e)))?;
        
        let proj_dir = temp_dir.path().join(format!("proj_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&proj_dir).await;

        let cargo_toml = proj_dir.join("Cargo.toml");
        let manifest = r#"[package]
name = "kowalski_rust_exec"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        fs::write(&cargo_toml, manifest)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create Cargo.toml: {}", e)))?;

        let src_dir = proj_dir.join("src");
        let _ = fs::create_dir_all(&src_dir).await;
        let main_file = src_dir.join("main.rs");

        let main_content = format!("fn main() {{\n{}\n}}", code);
        fs::write(&main_file, &main_content)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write main.rs: {}", e)))?;

        let child = Command::new("cargo")
            .arg("run")
            .arg("--manifest-path")
            .arg(&cargo_toml)
            .arg("--release")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to spawn Rust: {}", e)))?;

        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(result) => result.map_err(|e| {
                RLMError::ExecutionError(format!("Failed to wait for Rust: {}", e))
            })?,
            Err(_) => {
                return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() {
            return Err(RLMError::REPLError(format!(
                "Rust compilation/execution failed:\n{}",
                stderr
            )));
        }

        Ok(if stdout.is_empty() && stderr.is_empty() {
            "(no output)".to_string()
        } else {
            stdout
        })
    }

    fn language(&self) -> &str {
        "rust"
    }
}

impl JavaREPL {
    pub fn new() -> Self {
        JavaREPL {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for JavaREPL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl REPLExecutor for JavaREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp dir: {}", e)))?;

        let uuid = Uuid::new_v4().to_string().replace("-", "");
        let class_name = format!("Kowalski{}", &uuid[0..8]);
        let java_file = temp_dir.path().join(format!("{}.java", class_name));

        let java_code = format!(
            "public class {} {{\n    public static void main(String[] args) {{\n        {}\n    }}\n}}",
            class_name, code
        );

        fs::write(&java_file, &java_code)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write Java file: {}", e)))?;

        let javac_child = Command::new("javac")
            .arg(&java_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to spawn javac: {}", e)))?;

        let compile_output = match tokio::time::timeout(self.timeout, javac_child.wait_with_output()).await {
            Ok(result) => result.map_err(|e| {
                RLMError::ExecutionError(format!("Failed to wait for javac: {}", e))
            })?,
            Err(_) => {
                return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
            }
        };

        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr).to_string();
            return Err(RLMError::REPLError(format!("Java compilation failed:\n{}", stderr)));
        }

        let java_child = Command::new("java")
            .arg("-cp")
            .arg(temp_dir.path())
            .arg(&class_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to spawn java: {}", e)))?;

        let output = match tokio::time::timeout(self.timeout, java_child.wait_with_output()).await {
            Ok(result) => result.map_err(|e| {
                RLMError::ExecutionError(format!("Failed to wait for java: {}", e))
            })?,
            Err(_) => {
                return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() {
            return Err(RLMError::REPLError(format!(
                "Java execution failed:\n{}",
                stderr
            )));
        }

        Ok(if stdout.is_empty() && stderr.is_empty() {
            "(no output)".to_string()
        } else {
            stdout
        })
    }

    fn language(&self) -> &str {
        "java"
    }
}

impl BashREPL {
    pub fn new() -> Self {
        BashREPL {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for BashREPL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl REPLExecutor for BashREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp dir: {}", e)))?;

        let bash_file = temp_dir.path().join(format!("{}.sh", Uuid::new_v4()));

        fs::write(&bash_file, code)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write bash script: {}", e)))?;

        let child = Command::new("bash")
            .arg(&bash_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to spawn bash: {}", e)))?;

        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(result) => result.map_err(|e| {
                RLMError::ExecutionError(format!("Failed to wait for bash: {}", e))
            })?,
            Err(_) => {
                return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() {
            return Err(RLMError::REPLError(format!(
                "Bash execution failed:\n{}",
                stderr
            )));
        }

        Ok(if stdout.is_empty() && stderr.is_empty() {
            "(no output)".to_string()
        } else {
            stdout
        })
    }

    fn language(&self) -> &str {
        "bash"
    }
}

impl JavaScriptREPL {
    pub fn new() -> Self {
        JavaScriptREPL {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for JavaScriptREPL {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl REPLExecutor for JavaScriptREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp dir: {}", e)))?;

        let js_file = temp_dir.path().join(format!("{}.js", Uuid::new_v4()));

        fs::write(&js_file, code)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write JS file: {}", e)))?;

        let mut child = Command::new("node")
            .arg(&js_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RLMError::ExecutionError(format!("Failed to spawn Node.js: {}", e)))?;

        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(result) => result.map_err(|e| {
                RLMError::ExecutionError(format!("Failed to wait for Node.js: {}", e))
            })?,
            Err(_) => {
                return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() {
            return Err(RLMError::REPLError(format!(
                "JavaScript execution failed:\n{}",
                stderr
            )));
        }

        Ok(if stdout.is_empty() && stderr.is_empty() {
            "(no output)".to_string()
        } else {
            stdout
        })
    }

    fn language(&self) -> &str {
        "javascript"
    }
}

/// Factory for creating REPL executors
pub struct REPLExecutorFactory;

impl REPLExecutorFactory {
    /// Create a REPL executor for the given language
    pub fn create(language: &str) -> RLMResult<Box<dyn REPLExecutor>> {
        match language.to_lowercase().as_str() {
            "python" | "py" => Ok(Box::new(PythonREPL::new())),
            "rust" | "rs" => Ok(Box::new(RustREPL::new())),
            "java" => Ok(Box::new(JavaREPL::new())),
            "bash" | "sh" | "shell" => Ok(Box::new(BashREPL::new())),
            "javascript" | "js" => Ok(Box::new(JavaScriptREPL::new())),
            _ => Err(RLMError::ExecutionError(format!(
                "Unsupported language: {}",
                language
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]  // Requires Python to be installed
    async fn test_python_simple() {
        let executor = PythonREPL::new();
        let code = "print('hello')";
        let output = executor.execute(code).await.unwrap();
        assert!(output.contains("hello"));
    }

    #[tokio::test]
    #[ignore]  // Requires Rust to be installed
    async fn test_rust_simple() {
        let executor = RustREPL::new();
        let code = r#"println!("hello from rust");"#;
        let output = executor.execute(code).await.unwrap();
        assert!(output.contains("hello from rust"));
    }

    #[tokio::test]
    #[ignore]  // Requires Java to be installed
    async fn test_java_simple() {
        let executor = JavaREPL::new();
        let code = r#"System.out.println("hello from java");"#;
        let output = executor.execute(code).await.unwrap();
        assert!(output.contains("hello from java"));
    }

    #[tokio::test]
    #[ignore]  // Requires bash to be installed
    async fn test_bash_simple() {
        let executor = BashREPL::new();
        let code = "echo 'hello from bash'";
        let output = executor.execute(code).await.unwrap();
        assert!(output.contains("hello from bash"));
    }

    #[tokio::test]
    #[ignore]  // Requires Node to be installed
    async fn test_javascript_simple() {
        let executor = JavaScriptREPL::new();
        let code = "console.log('hello from javascript');";
        let output = executor.execute(code).await.unwrap();
        assert!(output.contains("hello from javascript"));
    }

    #[test]
    fn test_factory_python() {
        let executor = REPLExecutorFactory::create("python").unwrap();
        assert_eq!(executor.language(), "python");
    }

    #[test]
    fn test_factory_rust() {
        let executor = REPLExecutorFactory::create("rust").unwrap();
        assert_eq!(executor.language(), "rust");
    }

    #[test]
    fn test_factory_java() {
        let executor = REPLExecutorFactory::create("java").unwrap();
        assert_eq!(executor.language(), "java");
    }

    #[test]
    fn test_factory_bash() {
        let executor = REPLExecutorFactory::create("bash").unwrap();
        assert_eq!(executor.language(), "bash");
    }

    #[test]
    fn test_factory_javascript() {
        let executor = REPLExecutorFactory::create("javascript").unwrap();
        assert_eq!(executor.language(), "javascript");
    }

    #[test]
    fn test_factory_unsupported() {
        let result = REPLExecutorFactory::create("cpp");
        assert!(result.is_err());
    }
}
