use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub runner: String,
    pub command: String,
    pub passed: bool,
    pub exit_code: i32,
    pub executed_at: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestRunner {
    Cargo,
    Pytest,
    Npm,
    Go,
    #[allow(dead_code)]
    Custom(String),
}

impl TestRunner {
    pub fn display_name(&self) -> &str {
        match self {
            TestRunner::Cargo => "cargo test",
            TestRunner::Pytest => "pytest",
            TestRunner::Npm => "npm test",
            TestRunner::Go => "go test ./...",
            TestRunner::Custom(c) => c.as_str(),
        }
    }
}

/// Detect applicable test runner based on project files
pub fn detect_test_runner(root: &Path) -> Option<TestRunner> {
    if root.join("Cargo.toml").exists() {
        Some(TestRunner::Cargo)
    } else if root.join("pyproject.toml").exists()
        || root.join("pytest.ini").exists()
        || root.join("requirements.txt").exists()
    {
        Some(TestRunner::Pytest)
    } else if root.join("package.json").exists() {
        Some(TestRunner::Npm)
    } else if root.join("go.mod").exists() {
        Some(TestRunner::Go)
    } else {
        None
    }
}

/// Execute tests and cache results in .aiflow/.cache/test_results.json
pub fn run_tests(root: &Path) -> Result<TestResult, String> {
    let runner = detect_test_runner(root)
        .ok_or_else(|| "No recognized test runner found (Cargo.toml, package.json, pytest, go.mod)".to_string())?;

    let (program, args) = match &runner {
        TestRunner::Cargo => ("cargo", vec!["test", "--", "--nocapture"]),
        TestRunner::Pytest => ("pytest", vec![]),
        TestRunner::Npm => ("npm", vec!["test"]),
        TestRunner::Go => ("go", vec!["test", "./..."]),
        TestRunner::Custom(cmd) => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Err("Empty custom test command".to_string());
            }
            return run_command_direct(root, parts[0], &parts[1..], &runner.display_name());
        }
    };

    run_command_direct(root, program, &args, runner.display_name())
}

fn run_command_direct(root: &Path, program: &str, args: &[&str], display: &str) -> Result<TestResult, String> {
    println!("Running test suite: {} ...\n", display);

    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| format!("Failed to spawn test runner '{}': {}", program, e))?;

    let exit_code = status.code().unwrap_or(-1);
    let passed = status.success();
    let now = chrono::Utc::now().to_rfc3339();

    let result = TestResult {
        runner: program.to_string(),
        command: display.to_string(),
        passed,
        exit_code,
        executed_at: now,
        message: if passed {
            "All tests passed successfully.".to_string()
        } else {
            format!("Tests failed with exit code {}.", exit_code)
        },
    };

    let _ = save_cached_test_result(root, &result);
    Ok(result)
}

/// Path to cached test results: .aiflow/.cache/test_results.json
pub fn cached_test_result_path(root: &Path) -> std::path::PathBuf {
    root.join(".aiflow").join(".cache").join("test_results.json")
}

pub fn load_cached_test_result(root: &Path) -> Option<TestResult> {
    let path = cached_test_result_path(root);
    if path.exists() {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_cached_test_result(root: &Path, result: &TestResult) -> Result<(), String> {
    let path = cached_test_result_path(root);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(result)
        .map_err(|e| format!("Failed serializing test result: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Failed writing test result cache: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cargo_runner() {
        let root = Path::new(".");
        let runner = detect_test_runner(root);
        assert_eq!(runner, Some(TestRunner::Cargo));
    }

    #[test]
    fn test_serialize_test_result() {
        let res = TestResult {
            runner: "cargo".to_string(),
            command: "cargo test".to_string(),
            passed: true,
            exit_code: 0,
            executed_at: "2026-10-01T00:00:00Z".to_string(),
            message: "All tests passed".to_string(),
        };
        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("cargo test"));
        let decoded: TestResult = serde_json::from_str(&json).unwrap();
        assert!(decoded.passed);
    }
}
