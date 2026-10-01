use crate::domain::{ProjectConfig, ProjectState, TaskItem, WorkflowConfig};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse/serialize error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("AIFlow not initialized. Run 'aiflow init' first.")]
    NotInitialized,
}

pub fn aiflow_dir(root: &Path) -> PathBuf {
    root.join(".aiflow")
}

pub fn is_initialized(root: &Path) -> bool {
    aiflow_dir(root).join("project.yaml").exists()
}

/// Initialize the .aiflow directory with standard templates
pub fn init_project(
    root: &Path,
    preset: &str,
    project_name: &str,
    language: &str,
    stack: &str,
) -> Result<(), StorageError> {
    let dir = aiflow_dir(root);
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join(".cache"))?;

    // 1. project.yaml
    let project_cfg = ProjectConfig::new(project_name, language, stack);
    let project_yaml = serde_yaml::to_string(&project_cfg)?;
    fs::write(dir.join("project.yaml"), project_yaml)?;

    // 2. workflow.yaml
    let workflow_cfg = match preset {
        "lean" => WorkflowConfig::lean_4_phase(),
        _ => WorkflowConfig::standard_5_phase(),
    };
    let workflow_yaml = serde_yaml::to_string(&workflow_cfg)?;
    fs::write(dir.join("workflow.yaml"), workflow_yaml)?;

    // 3. state.yaml
    let state = ProjectState::initial(&workflow_cfg.initial_phase);
    let state_yaml = serde_yaml::to_string(&state)?;
    fs::write(dir.join("state.yaml"), state_yaml)?;

    // 4. spec.md (Architecture & Requirements template)
    let spec_path = dir.join("spec.md");
    if !spec_path.exists() {
        let spec_content = format!(
            "# Specification: {}\n\n\
            ## 1. Problem Statement & Scope\n\
            <!-- Defined by Human Developer -->\n\
            Describe what problem this project solves and the core requirements.\n\n\
            ## 2. Technical Architecture & Decisions\n\
            <!-- Authored by Claude (Architect) -->\n\
            Describe the component layout, technology choices, and data flow.\n\n\
            ## 3. Human Approval\n\
            - [ ] Architecture approved by lead developer\n",
            project_name
        );
        fs::write(spec_path, spec_content)?;
    }

    // 5. tasks.md template
    let tasks_path = dir.join("tasks.md");
    if !tasks_path.exists() {
        let tasks_content = format!(
            "# Project Tasks: {}\n\n\
            ## Phase: Build\n\
            - [ ] TASK-01: Set up core project structure and entry point\n\
            - [ ] TASK-02: Implement primary domain logic and tests\n",
            project_name
        );
        fs::write(tasks_path, tasks_content)?;
    }

    // 6. Ensure .aiflow/.cache/ is ignored by git
    let gitignore_path = root.join(".gitignore");
    let mut gitignore_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    if !gitignore_content.contains(".aiflow/.cache/") {
        if !gitignore_content.is_empty() && !gitignore_content.ends_with('\n') {
            gitignore_content.push('\n');
        }
        gitignore_content.push_str("\n# AIFlow volatile cache\n.aiflow/.cache/\n");
        fs::write(&gitignore_path, gitignore_content)?;
    }

    Ok(())
}

/// Load project, workflow, and state files
pub fn load_project_bundle(
    root: &Path,
) -> Result<(ProjectConfig, WorkflowConfig, ProjectState), StorageError> {
    let dir = aiflow_dir(root);
    if !dir.join("project.yaml").exists() {
        return Err(StorageError::NotInitialized);
    }

    let project_yaml = fs::read_to_string(dir.join("project.yaml"))?;
    let project_cfg: ProjectConfig = serde_yaml::from_str(&project_yaml)?;

    let workflow_yaml = fs::read_to_string(dir.join("workflow.yaml"))?;
    let workflow_cfg: WorkflowConfig = serde_yaml::from_str(&workflow_yaml)?;

    let state_yaml = fs::read_to_string(dir.join("state.yaml"))?;
    let state: ProjectState = serde_yaml::from_str(&state_yaml)?;

    Ok((project_cfg, workflow_cfg, state))
}

/// Save state.yaml
pub fn save_state(root: &Path, state: &ProjectState) -> Result<(), StorageError> {
    let dir = aiflow_dir(root);
    let state_yaml = serde_yaml::to_string(state)?;
    fs::write(dir.join("state.yaml"), state_yaml)?;
    Ok(())
}

/// Save project.yaml
pub fn save_project_config(root: &Path, config: &ProjectConfig) -> Result<(), StorageError> {
    let dir = aiflow_dir(root);
    let project_yaml = serde_yaml::to_string(config)?;
    fs::write(dir.join("project.yaml"), project_yaml)?;
    Ok(())
}

/// Parse task items from .aiflow/tasks.md
pub fn load_tasks(root: &Path) -> Result<Vec<TaskItem>, StorageError> {
    let tasks_path = aiflow_dir(root).join("tasks.md");
    if !tasks_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(tasks_path)?;
    let mut tasks = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [") {
            let completed = trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]");
            let remainder = trimmed.trim_start_matches("- [ ] ").trim_start_matches("- [x] ").trim_start_matches("- [X] ");
            let is_active = remainder.to_lowercase().contains("(active)");

            // Try to split on ":" to get TASK-ID
            let parts: Vec<&str> = remainder.splitn(2, ':').collect();
            let (id, title) = if parts.len() == 2 {
                (parts[0].trim().to_string(), parts[1].trim().to_string())
            } else {
                (format!("TASK-{:02}", tasks.len() + 1), remainder.to_string())
            };

            tasks.push(TaskItem {
                id,
                title,
                completed,
                is_active,
            });
        }
    }

    Ok(tasks)
}

/// Mark a specific task as done in .aiflow/tasks.md
pub fn mark_task_done(root: &Path, task_id: &str) -> Result<bool, StorageError> {
    let tasks_path = aiflow_dir(root).join("tasks.md");
    if !tasks_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&tasks_path)?;
    let mut updated_lines = Vec::new();
    let mut found = false;

    for line in content.lines() {
        if line.contains(task_id) && line.trim().starts_with("- [ ]") {
            let replaced = line.replace("- [ ]", "- [x]");
            // Remove (Active) tag if present
            let cleaned = replaced.replace("(Active)", "").replace("(active)", "");
            updated_lines.push(cleaned.trim_end().to_string());
            found = true;
        } else {
            updated_lines.push(line.to_string());
        }
    }

    if found {
        fs::write(tasks_path, updated_lines.join("\n") + "\n")?;
    }

    Ok(found)
}

/// Add a new task to .aiflow/tasks.md
pub fn add_task(root: &Path, title: &str) -> Result<String, StorageError> {
    let tasks_path = aiflow_dir(root).join("tasks.md");
    let current_tasks = load_tasks(root)?;
    let next_id = format!("TASK-{:02}", current_tasks.len() + 1);

    let line = format!("- [ ] {}: {}\n", next_id, title);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&tasks_path)?;
    file.write_all(line.as_bytes())?;

    Ok(next_id)
}
