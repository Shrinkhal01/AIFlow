use crate::domain::{GitInfo, Phase, ProjectConfig, ProjectState, Role, TaskItem, WorkflowConfig};
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

/// Compute the next logical action based on current state, git signals, tasks, and active AI stack
pub fn compute_next_action(
    project: &ProjectConfig,
    _workflow: &WorkflowConfig,
    state: &ProjectState,
    git: &GitInfo,
    tasks: &[TaskItem],
    repo_root: &Path,
) -> NextAction {
    let current_phase = state.current_phase.as_str();

    let architect_agent = project
        .roles
        .get("architect")
        .map(|s| s.as_str())
        .unwrap_or("claude-3-7-sonnet");

    let implementer_agent = project
        .roles
        .get("implementer")
        .map(|s| s.as_str())
        .unwrap_or("codex-cursor");

    let is_chatgpt_architect = architect_agent.contains("chatgpt");
    let is_claude_coder = implementer_agent.contains("claude-code");

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
            } else if is_chatgpt_architect {
                NextAction {
                    summary: "Review specification drafted with ChatGPT".to_string(),
                    assigned_role: Role::Approver,
                    recommended_command: "aiflow phase complete".to_string(),
                    explanation: "Draft architecture & scope in .aiflow/spec.md with ChatGPT (o3/4o), then approve to advance.".to_string(),
                }
            } else {
                NextAction {
                    summary: "Review specification drafted with Claude".to_string(),
                    assigned_role: Role::Approver,
                    recommended_command: "aiflow phase complete".to_string(),
                    explanation: "Draft architecture & scope in .aiflow/spec.md with Claude 3.7, then approve to advance.".to_string(),
                }
            }
        }
        "plan" => {
            if tasks.is_empty() {
                if is_chatgpt_architect {
                    NextAction {
                        summary: "Generate task list in .aiflow/tasks.md using ChatGPT".to_string(),
                        assigned_role: Role::Architect,
                        recommended_command: "aiflow task add \"TASK-01: Feature scaffold\"".to_string(),
                        explanation: "Deconstruct spec.md into modular task items using ChatGPT (o3 / GPT-4o).".to_string(),
                    }
                } else {
                    NextAction {
                        summary: "Generate task list in .aiflow/tasks.md using Claude".to_string(),
                        assigned_role: Role::Architect,
                        recommended_command: "aiflow task add \"TASK-01: Feature scaffold\"".to_string(),
                        explanation: "Deconstruct spec.md into modular task items using Claude 3.7 Sonnet.".to_string(),
                    }
                }
            } else if is_claude_coder {
                NextAction {
                    summary: format!("Begin Build phase with Claude Code ({} tasks defined)", tasks.len()),
                    assigned_role: Role::Implementer,
                    recommended_command: "aiflow phase next".to_string(),
                    explanation: "Tasks are planned and ready for implementation by Claude Code in your terminal.".to_string(),
                }
            } else {
                NextAction {
                    summary: format!("Begin Build phase with Codex/Cursor ({} tasks defined)", tasks.len()),
                    assigned_role: Role::Implementer,
                    recommended_command: "aiflow phase next".to_string(),
                    explanation: "Tasks are planned and ready for implementation by Codex / Cursor IDE.".to_string(),
                }
            }
        }
        "build" => {
            let pending_tasks: Vec<&TaskItem> = tasks.iter().filter(|t| !t.completed).collect();
            if let Some(first_pending) = pending_tasks.first() {
                if is_claude_coder {
                    NextAction {
                        summary: format!("Implement {}: {} with Claude Code", first_pending.id, first_pending.title),
                        assigned_role: Role::Implementer,
                        recommended_command: format!("claude \"Implement {}\"", first_pending.title),
                        explanation: format!("Run 'claude' in terminal to author {}. Mark done with 'aiflow task done {}'.", first_pending.id, first_pending.id),
                    }
                } else {
                    NextAction {
                        summary: format!("Implement {}: {} in Cursor / Codex", first_pending.id, first_pending.title),
                        assigned_role: Role::Implementer,
                        recommended_command: format!("aiflow task done {}", first_pending.id),
                        explanation: format!("Author code in your IDE for {}. Test and mark done once verified.", first_pending.id),
                    }
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
                    explanation: "Build phase complete. Ready to run test suites and adversarial review with Gemini.".to_string(),
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
                    summary: "Perform adversarial review with Gemini".to_string(),
                    assigned_role: Role::Reviewer,
                    recommended_command: "aiflow phase complete".to_string(),
                    explanation: "Prompt Gemini 2.5 Pro with code diff for edge-case and security audit. Once approved, advance to Ship.".to_string(),
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
