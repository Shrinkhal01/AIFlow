use crate::fsm::compute_next_action;
use crate::git::inspect_git;
use crate::storage::{is_initialized, load_project_bundle, load_tasks};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub name: String,
    pub path: String,
    pub language: String,
    pub is_aiflow_initialized: bool,
    pub phase: Option<String>,
    pub active_role: Option<String>,
    pub git_branch: String,
    pub git_modified: usize,
    pub git_untracked: usize,
    pub tasks_done: usize,
    pub tasks_total: usize,
    pub next_action: String,
}

/// Directories that should be pruned immediately during search
fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | "vendor"
            | ".cache"
            | "venv"
            | ".venv"
            | ".cargo"
            | "dist"
            | "build"
            | ".idea"
            | ".vscode"
    )
}

/// Discover all Git / AIFlow repositories under the specified root paths
pub fn discover_projects(roots: &[PathBuf], max_depth: usize) -> Vec<ProjectSummary> {
    let mut discovered_dirs = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        let walker = WalkDir::new(root)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if let Some(file_name) = e.file_name().to_str() {
                    if e.file_type().is_dir() && should_skip_dir(file_name) {
                        return false;
                    }
                }
                true
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let path = entry.path();
                // Check if this directory is a repo root
                if path.join(".git").exists() || path.join(".aiflow").exists() {
                    // Check if we haven't already added this path or a parent
                    if !discovered_dirs.iter().any(|p: &PathBuf| path.starts_with(p)) {
                        discovered_dirs.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    discovered_dirs.sort();
    discovered_dirs.dedup();

    discovered_dirs
        .into_iter()
        .map(|path| summarize_project(&path))
        .collect()
}

/// Extract summary metrics for a given repository directory
fn summarize_project(path: &Path) -> ProjectSummary {
    let dir_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let git = inspect_git(path).unwrap_or_default();

    if is_initialized(path) {
        if let Ok((project, workflow, state)) = load_project_bundle(path) {
            let tasks = load_tasks(path).unwrap_or_default();
            let tasks_total = tasks.len();
            let tasks_done = tasks.iter().filter(|t| t.completed).count();
            let next = compute_next_action(&project, &workflow, &state, &git, &tasks, path);

            return ProjectSummary {
                name: project.name,
                path: path.display().to_string(),
                language: project.language,
                is_aiflow_initialized: true,
                phase: Some(state.current_phase),
                active_role: Some(next.assigned_role.to_string()),
                git_branch: git.branch,
                git_modified: git.modified_count,
                git_untracked: git.untracked_count,
                tasks_done,
                tasks_total,
                next_action: next.summary,
            };
        }
    }

    // Uninitialized Git repository fallback
    let detected_lang = crate::detect_language(path);
    ProjectSummary {
        name: dir_name,
        path: path.display().to_string(),
        language: detected_lang.to_string(),
        is_aiflow_initialized: false,
        phase: None,
        active_role: None,
        git_branch: git.branch,
        git_modified: git.modified_count,
        git_untracked: git.untracked_count,
        tasks_done: 0,
        tasks_total: 0,
        next_action: "Run 'aiflow init'".to_string(),
    }
}

/// Render a terminal table for the discovered projects
pub fn render_projects_table(projects: &[ProjectSummary], show_all: bool, phase_filter: Option<&str>) {
    let filtered: Vec<&ProjectSummary> = projects
        .iter()
        .filter(|p| {
            if !show_all && !p.is_aiflow_initialized {
                return false;
            }
            if let Some(filter) = phase_filter {
                if let Some(phase) = &p.phase {
                    return phase.eq_ignore_ascii_case(filter);
                } else {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        println!("\nNo matching projects found.");
        if !show_all {
            println!("(Tip: Pass '--all' to also show uninitialized git repositories)");
        }
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Project").fg(Color::Cyan),
            Cell::new("Lang").fg(Color::Cyan),
            Cell::new("Phase").fg(Color::Cyan),
            Cell::new("Git Branch (Changes)").fg(Color::Cyan),
            Cell::new("Tasks").fg(Color::Cyan),
            Cell::new("Next Action").fg(Color::Cyan),
        ]);

    for p in filtered {
        let (phase_cell, phase_color) = match &p.phase {
            Some(phase) => {
                let color = match phase.as_str() {
                    "specify" => Color::Cyan,
                    "plan" => Color::Yellow,
                    "build" => Color::Blue,
                    "verify" => Color::Magenta,
                    "ship" => Color::Green,
                    _ => Color::White,
                };
                (phase.to_uppercase(), color)
            }
            None => ("NOT INIT".to_string(), Color::DarkGrey),
        };

        let git_status_str = if p.git_branch.is_empty() {
            "-".to_string()
        } else if p.git_modified == 0 && p.git_untracked == 0 {
            format!("{} (clean)", p.git_branch)
        } else {
            format!(
                "{} ({}m, {}u)",
                p.git_branch, p.git_modified, p.git_untracked
            )
        };

        let git_color = if p.git_modified == 0 && p.git_untracked == 0 {
            Color::Green
        } else {
            Color::Yellow
        };

        let tasks_str = if p.is_aiflow_initialized {
            format!("{}/{} done", p.tasks_done, p.tasks_total)
        } else {
            "-".to_string()
        };

        table.add_row(vec![
            Cell::new(&p.name).fg(Color::White),
            Cell::new(&p.language).fg(Color::DarkGrey),
            Cell::new(phase_cell).fg(phase_color),
            Cell::new(git_status_str).fg(git_color),
            Cell::new(tasks_str),
            Cell::new(&p.next_action).fg(Color::White),
        ]);
    }

    println!("\n{}", table);
    println!("Total tracked projects: {}\n", projects.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_directories() {
        assert!(should_skip_dir(".git"));
        assert!(should_skip_dir("target"));
        assert!(should_skip_dir("node_modules"));
        assert!(should_skip_dir(".cache"));
        assert!(!should_skip_dir("src"));
        assert!(!should_skip_dir("docs"));
    }

    #[test]
    fn test_discover_current_project() {
        let roots = vec![PathBuf::from(".")];
        let projects = discover_projects(&roots, 1);
        assert!(!projects.is_empty());
        let aiflow = projects.iter().find(|p| p.name == "AIFlow");
        assert!(aiflow.is_some());
        assert!(aiflow.unwrap().is_aiflow_initialized);
    }
}
