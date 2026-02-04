use async_trait::async_trait;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use crate::error::{RLMError, RLMResult};
use uuid::Uuid;
use std::path::PathBuf;

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
    temp_dir: PathBuf,
}

/// Rust REPL Executor
pub struct RustREPL {
    timeout: Duration,
    temp_dir: PathBuf,
}

/// Java REPL Executor
pub struct JavaREPL {
    timeout: Duration,
    temp_dir: PathBuf,
}

/// Bash/Shell REPL Executor
pub struct BashREPL {
    timeout: Duration,
    temp_dir: PathBuf,
}

/// JavaScript REPL Executor
pub struct JavaScriptREPL {
    timeout: Duration,
    temp_dir: PathBuf,
}

impl PythonREPL {
    pub fn new() -> Self {
        PythonREPL {
            timeout: Duration::from_secs(30),
            temp_dir: PathBuf::from("/tmp/kowalski_python"),
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
        // Create temp directory if it doesn't exist
        let _ = fs::create_dir_all(&self.temp_dir).await;

        // Generate unique temp file
        let temp_file = self.temp_dir.join(format!("{}.py", Uuid::new_v4()));

        // Write code to temp file
        let mut file = fs::File::create(&temp_file)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to create temp file: {}", e)))?;

        file.write_all(code.as_bytes())
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write code: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to sync file: {}", e)))?;

        // Execute Python
        let output = tokio::time::timeout(
            self.timeout,
            Command::new("python3")
                .arg(&temp_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| {
            RLMError::REPLTimeout(self.timeout.as_millis() as u64)
        })?
        .map_err(|e| RLMError::ExecutionError(format!("Failed to execute Python: {}", e)))?;

        // Cleanup temp file
        let _ = fs::remove_file(&temp_file).await;

        // Combine stdout and stderr
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
            timeout: Duration::from_secs(60),
            temp_dir: PathBuf::from("/tmp/kowalski_rust"),
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
        // Create temp directory
        let _ = fs::create_dir_all(&self.temp_dir).await;

        let proj_dir = self.temp_dir.join(format!("proj_{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&proj_dir).await;

        // Create Cargo.toml
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

        // Create src/main.rs
        let src_dir = proj_dir.join("src");
        let _ = fs::create_dir_all(&src_dir).await;
        let main_file = src_dir.join("main.rs");

        let main_content = format!("fn main() {{\n{}\n}}", code);
        fs::write(&main_file, &main_content)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write main.rs: {}", e)))?;

        // Compile and run
        let output = tokio::time::timeout(
            self.timeout,
            Command::new("cargo")
                .arg("run")
                .arg("--manifest-path")
                .arg(&cargo_toml)
                .arg("--release")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| {
            RLMError::REPLTimeout(self.timeout.as_millis() as u64)
        })?
        .map_err(|e| RLMError::ExecutionError(format!("Failed to execute Rust: {}", e)))?;

        // Cleanup
        let _ = fs::remove_dir_all(&proj_dir).await;

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
            temp_dir: PathBuf::from("/tmp/kowalski_java"),
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
        // Create temp directory
        let _ = fs::create_dir_all(&self.temp_dir).await;

        let uuid = Uuid::new_v4().to_string().replace("-", "");
        let class_name = format!("Kowalski{}", &uuid[0..8]);
        let java_file = self.temp_dir.join(format!("{}.java", class_name));

        // Wrap code in class
        let java_code = format!(
            "public class {} {{\n    public static void main(String[] args) {{\n        {}\n    }}\n}}",
            class_name, code
        );

        fs::write(&java_file, &java_code)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write Java file: {}", e)))?;

        // Compile
        tokio::time::timeout(
            self.timeout,
            Command::new("javac")
                .arg(&java_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| {
            RLMError::REPLTimeout(self.timeout.as_millis() as u64)
        })?
        .map_err(|e| RLMError::ExecutionError(format!("Failed to compile Java: {}", e)))?;

        // Run
        let output = tokio::time::timeout(
            self.timeout,
            Command::new("java")
                .arg("-cp")
                .arg(&self.temp_dir)
                .arg(&class_name)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| {
            RLMError::REPLTimeout(self.timeout.as_millis() as u64)
        })?
        .map_err(|e| RLMError::ExecutionError(format!("Failed to run Java: {}", e)))?;

        // Cleanup
        let _ = fs::remove_file(&java_file).await;
        let _ = fs::remove_file(self.temp_dir.join(format!("{}.class", class_name))).await;

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
            temp_dir: PathBuf::from("/tmp/kowalski_bash"),
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
        // Create temp directory if needed
        let _ = fs::create_dir_all(&self.temp_dir).await;

        let bash_file = self.temp_dir.join(format!("{}.sh", Uuid::new_v4()));

        // Write script
        fs::write(&bash_file, code)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write bash script: {}", e)))?;

        // Execute
        let output = tokio::time::timeout(
            self.timeout,
            Command::new("bash")
                .arg(&bash_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| {
            RLMError::REPLTimeout(self.timeout.as_millis() as u64)
        })?
        .map_err(|e| RLMError::ExecutionError(format!("Failed to execute bash: {}", e)))?;

        // Cleanup
        let _ = fs::remove_file(&bash_file).await;

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
            temp_dir: PathBuf::from("/tmp/kowalski_js"),
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
        // Create temp directory if needed
        let _ = fs::create_dir_all(&self.temp_dir).await;

        let js_file = self.temp_dir.join(format!("{}.js", Uuid::new_v4()));

        // Write code
        fs::write(&js_file, code)
            .await
            .map_err(|e| RLMError::ExecutionError(format!("Failed to write JS file: {}", e)))?;

        // Execute with node
        let output = tokio::time::timeout(
            self.timeout,
            Command::new("node")
                .arg(&js_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| {
            RLMError::REPLTimeout(self.timeout.as_millis() as u64)
        })?
        .map_err(|e| RLMError::ExecutionError(format!("Failed to execute JavaScript: {}", e)))?;

        // Cleanup
        let _ = fs::remove_file(&js_file).await;

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
