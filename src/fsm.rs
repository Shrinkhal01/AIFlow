use crate::domain::{GitInfo, Phase, ProjectState, Role, TaskItem, WorkflowConfig};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct NextAction {
    pub summary: String,
    pub assigned_role: Role,
    pub recommended_command: String,
    pub explanation: String,
}

/// Look up a phase by ID in the workflow configuration
pub fn find_phase<'a>(workflow: &'a WorkflowConfig, phase_id: &str) -> Option<&'a Phase> {
    workflow.phases.iter().find(|p| p.id == phase_id)
}

/// Compute the next logical action based on current state, git signals, and tasks
pub fn compute_next_action(
    _workflow: &WorkflowConfig,
    state: &ProjectState,
    git: &GitInfo,
    tasks: &[TaskItem],
    repo_root: &Path,
) -> NextAction {
    let current_phase = state.current_phase.as_str();

    match current_phase {
        "specify" => {
            let spec_path = repo_root.join(".aiflow/spec.md");
            if !spec_path.exists() {
                NextAction {
                    summary: "Create .aiflow/spec.md".to_string(),
                    assigned_role: Role::Architect,
                    recommended_command: "aiflow init".to_string(),
                    explanation: "Project specification document is missing.".to_string(),
                }
            } else {
                NextAction {
                    summary: "Review and approve architectural specification".to_string(),
                    assigned_role: Role::Approver,
                    recommended_command: "aiflow phase complete".to_string(),
                    explanation: "Draft specification in .aiflow/spec.md with Claude, then approve it to advance to Planning.".to_string(),
                }
            }
        }
        "plan" => {
            if tasks.is_empty() {
                NextAction {
                    summary: "Define initial tasks in .aiflow/tasks.md".to_string(),
                    assigned_role: Role::Architect,
                    recommended_command: "aiflow task add \"Initial feature implementation\"".to_string(),
                    explanation: "Break the specification down into checkable work items for the Build phase.".to_string(),
                }
            } else {
                NextAction {
                    summary: format!("Begin Build phase ({} tasks defined)", tasks.len()),
                    assigned_role: Role::Implementer,
                    recommended_command: "aiflow phase next".to_string(),
                    explanation: "Tasks are planned and ready for implementation by Codex/Cursor.".to_string(),
                }
            }
        }
        "build" => {
            let pending_tasks: Vec<&TaskItem> = tasks.iter().filter(|t| !t.completed).collect();
            if let Some(first_pending) = pending_tasks.first() {
                NextAction {
                    summary: format!("Implement {}: {}", first_pending.id, first_pending.title),
                    assigned_role: Role::Implementer,
                    recommended_command: format!("aiflow task done {}", first_pending.id),
                    explanation: "Code authoring with Codex. Mark task done once working.".to_string(),
                }
            } else if tasks.is_empty() {
                NextAction {
                    summary: "Add implementation tasks".to_string(),
                    assigned_role: Role::Implementer,
                    recommended_command: "aiflow task add \"Implement core feature\"".to_string(),
                    explanation: "No tasks found in .aiflow/tasks.md.".to_string(),
                }
            } else {
                NextAction {
                    summary: "All implementation tasks completed → advance to Verify".to_string(),
                    assigned_role: Role::Tester,
                    recommended_command: "aiflow phase next".to_string(),
                    explanation: "Build phase complete. Ready to run test suites and adversarial code review with Gemini.".to_string(),
                }
            }
        }
        "verify" => {
            if let Some(test_res) = crate::tester::load_cached_test_result(repo_root) {
                if !test_res.passed {
                    return NextAction {
                        summary: format!("Fix failing tests ({})", test_res.command),
                        assigned_role: Role::Implementer,
                        recommended_command: "aiflow test run".to_string(),
                        explanation: format!("Test run failed with exit code {}. Resolve errors before proceeding.", test_res.exit_code),
                    };
                }
            } else if crate::tester::detect_test_runner(repo_root).is_some() {
                return NextAction {
                    summary: "Run project test suite".to_string(),
                    assigned_role: Role::Tester,
                    recommended_command: "aiflow test run".to_string(),
                    explanation: "Test runner detected. Run tests to record verification evidence.".to_string(),
                };
            }

            if git.modified_count > 0 {
                NextAction {
                    summary: "Commit or stash modified files before final review".to_string(),
                    assigned_role: Role::Reviewer,
                    recommended_command: "git status && git commit -m \"verify: pass test suite\"".to_string(),
                    explanation: format!("{} modified files detected in working tree.", git.modified_count),
                }
            } else {
                NextAction {
                    summary: "Perform adversarial review & test verification with Gemini".to_string(),
                    assigned_role: Role::Reviewer,
                    recommended_command: "aiflow phase complete".to_string(),
                    explanation: "Verify test coverage and edge cases. Once approved, advance to Ship.".to_string(),
                }
            }
        }
        "ship" => {
            NextAction {
                summary: "Update documentation and tag release".to_string(),
                assigned_role: Role::Approver,
                recommended_command: "git tag -a v0.1.0 -m \"Release v0.1.0\"".to_string(),
                explanation: "Final human signoff: verify README, changelog, and push git tags.".to_string(),
            }
        }
        _ => NextAction {
            summary: "Check project health".to_string(),
            assigned_role: Role::Approver,
            recommended_command: "aiflow doctor".to_string(),
            explanation: format!("Unknown or custom phase: {}", current_phase),
        },
    }
}
