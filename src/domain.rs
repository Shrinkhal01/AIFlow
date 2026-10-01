use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cognitive AI roles supported in AIFlow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Architect,
    Implementer,
    Reviewer,
    Tester,
    Approver,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Architect => write!(f, "Architect (Claude)"),
            Role::Implementer => write!(f, "Implementer (Codex)"),
            Role::Reviewer => write!(f, "Reviewer (Gemini)"),
            Role::Tester => write!(f, "Tester (Gemini)"),
            Role::Approver => write!(f, "Approver (Human)"),
        }
    }
}

/// A workflow phase in the state machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Phase {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub required_artifacts: Vec<String>,
    #[serde(default)]
    pub requires_human_approval: bool,
    pub next_phase: Option<String>,
}

/// The workflow definition (.aiflow/workflow.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub version: String,
    pub name: String,
    pub preset: String,
    pub initial_phase: String,
    pub phases: Vec<Phase>,
}

impl WorkflowConfig {
    /// The default 5-phase workflow (Specify -> Plan -> Build -> Verify -> Ship)
    pub fn standard_5_phase() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Standard AI Workflow".to_string(),
            preset: "standard".to_string(),
            initial_phase: "specify".to_string(),
            phases: vec![
                Phase {
                    id: "specify".to_string(),
                    name: "Specify".to_string(),
                    role: "architect".to_string(),
                    required_artifacts: vec![".aiflow/spec.md".to_string()],
                    requires_human_approval: true,
                    next_phase: Some("plan".to_string()),
                },
                Phase {
                    id: "plan".to_string(),
                    name: "Plan".to_string(),
                    role: "architect".to_string(),
                    required_artifacts: vec![".aiflow/tasks.md".to_string()],
                    requires_human_approval: false,
                    next_phase: Some("build".to_string()),
                },
                Phase {
                    id: "build".to_string(),
                    name: "Build".to_string(),
                    role: "implementer".to_string(),
                    required_artifacts: vec![],
                    requires_human_approval: false,
                    next_phase: Some("verify".to_string()),
                },
                Phase {
                    id: "verify".to_string(),
                    name: "Verify".to_string(),
                    role: "tester".to_string(),
                    required_artifacts: vec![],
                    requires_human_approval: true,
                    next_phase: Some("ship".to_string()),
                },
                Phase {
                    id: "ship".to_string(),
                    name: "Ship".to_string(),
                    role: "approver".to_string(),
                    required_artifacts: vec![],
                    requires_human_approval: true,
                    next_phase: None,
                },
            ],
        }
    }

    /// The lean 4-phase workflow (Plan -> Build -> Verify -> Ship)
    pub fn lean_4_phase() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Lean AI Workflow".to_string(),
            preset: "lean".to_string(),
            initial_phase: "plan".to_string(),
            phases: vec![
                Phase {
                    id: "plan".to_string(),
                    name: "Plan".to_string(),
                    role: "architect".to_string(),
                    required_artifacts: vec![".aiflow/tasks.md".to_string()],
                    requires_human_approval: true,
                    next_phase: Some("build".to_string()),
                },
                Phase {
                    id: "build".to_string(),
                    name: "Build".to_string(),
                    role: "implementer".to_string(),
                    required_artifacts: vec![],
                    requires_human_approval: false,
                    next_phase: Some("verify".to_string()),
                },
                Phase {
                    id: "verify".to_string(),
                    name: "Verify".to_string(),
                    role: "tester".to_string(),
                    required_artifacts: vec![],
                    requires_human_approval: true,
                    next_phase: Some("ship".to_string()),
                },
                Phase {
                    id: "ship".to_string(),
                    name: "Ship".to_string(),
                    role: "approver".to_string(),
                    required_artifacts: vec![],
                    requires_human_approval: true,
                    next_phase: None,
                },
            ],
        }
    }
}

/// Project identity metadata (.aiflow/project.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub language: String,
    #[serde(default)]
    pub roles: HashMap<String, String>,
}

impl ProjectConfig {
    pub fn default_for(name: &str, language: &str) -> Self {
        let mut roles = HashMap::new();
        roles.insert("architect".to_string(), "claude-3-7-sonnet".to_string());
        roles.insert("implementer".to_string(), "codex".to_string());
        roles.insert("reviewer".to_string(), "gemini-2.5-pro".to_string());
        roles.insert("tester".to_string(), "gemini-2.5-pro".to_string());
        roles.insert("approver".to_string(), "human".to_string());

        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: format!("Project {} tracked by AIFlow", name),
            language: language.to_string(),
            roles,
        }
    }
}

/// Record of an approved/completed phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedPhase {
    pub phase: String,
    pub completed_at: String,
    pub approved_by: String,
}

/// Durable project state (.aiflow/state.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub version: String,
    pub current_phase: String,
    pub phase_started_at: String,
    #[serde(default)]
    pub completed_phases: Vec<CompletedPhase>,
    pub active_task_id: Option<String>,
}

impl ProjectState {
    pub fn initial(initial_phase: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            version: "1.0".to_string(),
            current_phase: initial_phase.to_string(),
            phase_started_at: now,
            completed_phases: Vec::new(),
            active_task_id: None,
        }
    }

    pub fn is_phase_completed(&self, phase_id: &str) -> bool {
        self.completed_phases.iter().any(|c| c.phase == phase_id)
    }
}

/// A parsed task item from .aiflow/tasks.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub is_active: bool,
}

/// Read-only Git inspection data
#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub branch: String,
    pub modified_count: usize,
    pub untracked_count: usize,
    pub last_commit_message: Option<String>,
}
