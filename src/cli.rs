use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "aiflow",
    author = "Shrinkhal",
    version = "0.1.0",
    about = "Local developer workflow and project-tracking CLI for AI-assisted development",
    long_about = "A local-first, Git-native developer control plane that tracks repository development phases, tasks, and next actions."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize AIFlow in the current repository
    Init(InitArgs),

    /// Display current project status, phase breadcrumb, and next action
    Status(StatusArgs),

    /// Display recommended next action and assigned AI role
    Next,

    /// Manage workflow phases
    Phase(PhaseArgs),

    /// Inspect and manage project tasks
    Task(TaskArgs),

    /// Run health check verifying repository structure and state
    Doctor,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Workflow preset to use: 'standard' (5 phases, default) or 'lean' (4 phases)
    #[arg(short, long, default_value = "standard")]
    pub preset: String,

    /// Project name override (defaults to current directory name)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Primary programming language (defaults to auto-detected)
    #[arg(short, long)]
    pub lang: Option<String>,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Output raw JSON instead of formatted terminal UI
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct PhaseArgs {
    #[command(subcommand)]
    pub action: PhaseSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum PhaseSubcommands {
    /// Show current phase details
    Status,
    /// Advance to the next phase in the workflow
    Next,
    /// Mark current phase as approved and advance
    Complete,
    /// Manually jump to a specific phase ID
    Set { phase_id: String },
}

#[derive(Args, Debug)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub action: Option<TaskSubcommands>,
}

#[derive(Subcommand, Debug)]
pub enum TaskSubcommands {
    /// List all tasks from .aiflow/tasks.md
    List,
    /// Mark a task as completed by ID (e.g. TASK-01)
    Done { task_id: String },
    /// Add a new task to .aiflow/tasks.md
    Add { title: String },
}
