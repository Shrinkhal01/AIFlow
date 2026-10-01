mod cli;
mod doctor;
mod domain;
mod fsm;
mod git;
mod storage;

use clap::Parser;
use cli::{Cli, Commands, PhaseSubcommands, TaskSubcommands};
use domain::CompletedPhase;
use fsm::{compute_next_action, find_phase};
use git::inspect_git;
use owo_colors::OwoColorize;
use std::env;
use std::path::{Path, PathBuf};
use storage::{add_task, init_project, is_initialized, load_project_bundle, load_tasks, mark_task_done, save_state};

fn main() {
    let args = Cli::parse();
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    match args.command.unwrap_or(Commands::Status(cli::StatusArgs { json: false })) {
        Commands::Init(init_args) => handle_init(&current_dir, &init_args),
        Commands::Status(status_args) => handle_status(&current_dir, status_args.json),
        Commands::Next => handle_next(&current_dir),
        Commands::Phase(phase_args) => handle_phase(&current_dir, &phase_args.action),
        Commands::Task(task_args) => handle_task(&current_dir, task_args.action),
        Commands::Doctor => {
            doctor::run_doctor(&current_dir);
        }
    }
}

/// Auto-detect project programming language from repository files
fn detect_language(root: &Path) -> &'static str {
    if root.join("Cargo.toml").exists() {
        "rust"
    } else if root.join("go.mod").exists() {
        "go"
    } else if root.join("package.json").exists() {
        "typescript"
    } else if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        "python"
    } else {
        "generic"
    }
}

fn handle_init(root: &Path, args: &cli::InitArgs) {
    if is_initialized(root) {
        println!("{}", "AIFlow is already initialized in this repository.".yellow());
        println!("Run '{}' to inspect project status.", "aiflow status".cyan());
        return;
    }

    let default_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let name = args.name.as_deref().unwrap_or(default_name);
    let detected_lang = detect_language(root);
    let lang = args.lang.as_deref().unwrap_or(detected_lang);

    match init_project(root, &args.preset, name, lang) {
        Ok(_) => {
            println!("\n{}", "🎉 Initialized AIFlow project successfully!".green().bold());
            println!("  Project Name:  {}", name.cyan());
            println!("  Language:      {}", lang.cyan());
            println!("  Preset:        {}", args.preset.magenta());
            println!("  Artifacts:     .aiflow/project.yaml, workflow.yaml, state.yaml, spec.md, tasks.md\n");
            println!("Next Steps:");
            println!("  1. View project status:    {}", "aiflow status".bold().cyan());
            println!("  2. See next action:        {}", "aiflow next".bold().cyan());
            println!("  3. Health check:           {}\n", "aiflow doctor".bold().cyan());
        }
        Err(e) => {
            eprintln!("{} Failed to initialize AIFlow: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn handle_status(root: &Path, json: bool) {
    if !is_initialized(root) {
        eprintln!("{}", "Error: AIFlow is not initialized in this repository.".red().bold());
        eprintln!("Run '{}' to initialize.", "aiflow init".cyan());
        std::process::exit(1);
    }

    let (project, workflow, state) = match load_project_bundle(root) {
        Ok(bundle) => bundle,
        Err(e) => {
            eprintln!("{} Failed loading project: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let git = inspect_git(root).unwrap_or_default();
    let tasks = load_tasks(root).unwrap_or_default();
    let next_action = compute_next_action(&workflow, &state, &git, &tasks, root);

    if json {
        let json_output = serde_json::json!({
            "project": project,
            "workflow_preset": workflow.preset,
            "current_phase": state.current_phase,
            "git": {
                "branch": git.branch,
                "modified_files": git.modified_count,
                "untracked_files": git.untracked_count,
                "last_commit": git.last_commit_message
            },
            "tasks_count": tasks.len(),
            "tasks_completed": tasks.iter().filter(|t| t.completed).count(),
            "next_action": {
                "summary": next_action.summary,
                "role": next_action.assigned_role.to_string(),
                "command": next_action.recommended_command
            }
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        return;
    }

    // Terminal Display
    println!("\nProject:      {} ({})", project.name.bold(), project.language.cyan());
    println!("Location:     {}", root.display().to_string().dimmed());
    if let Some(ref commit) = git.last_commit_message {
        println!("Last Commit:  {}", commit.dimmed());
    }

    // Workflow Breadcrumb
    println!("\n{}", "Workflow Progress:".bold());
    for (idx, phase) in workflow.phases.iter().enumerate() {
        let num = idx + 1;
        if state.is_phase_completed(&phase.id) {
            println!("  {} {}. {:<12} (Completed)", "✓".green(), num, phase.name);
        } else if phase.id == state.current_phase {
            println!(
                "  {} {}. {:<12} [ACTIVE - {}]",
                "→".cyan().bold(),
                num,
                phase.name.bold().cyan(),
                phase.role.yellow()
            );
        } else {
            println!("  {} {}. {:<12}", "○".dimmed(), num, phase.name.dimmed());
        }
    }

    // Git Status
    println!("\n{}", "Git Status:".bold());
    println!("  Branch:       {}", git.branch.cyan());
    println!(
        "  Working Tree: {} modified, {} untracked",
        git.modified_count, git.untracked_count
    );

    // Tasks Status
    let completed_count = tasks.iter().filter(|t| t.completed).count();
    println!("\n{}", "Tasks:".bold());
    println!("  Progress:     {}/{} completed", completed_count, tasks.len());
    if let Some(active) = tasks.iter().find(|t| !t.completed) {
        println!("  Active Task:  {}: {}", active.id.cyan(), active.title);
    }

    // Next Action
    println!("\n{}", "Next Logical Action:".bold().yellow());
    println!("  → {}", next_action.summary.bold());
    println!("  → Recommended Command: {}", next_action.recommended_command.cyan());
    println!("  → Assigned Role:       {}", next_action.assigned_role.to_string().magenta());
    println!("  → Context:             {}\n", next_action.explanation.dimmed());
}

fn handle_next(root: &Path) {
    if !is_initialized(root) {
        eprintln!("{}", "Error: AIFlow is not initialized. Run 'aiflow init'.".red());
        std::process::exit(1);
    }

    let (project, workflow, state) = match load_project_bundle(root) {
        Ok(bundle) => bundle,
        Err(e) => {
            eprintln!("{} Failed loading project: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };

    let git = inspect_git(root).unwrap_or_default();
    let tasks = load_tasks(root).unwrap_or_default();
    let next_action = compute_next_action(&workflow, &state, &git, &tasks, root);

    println!("\n{}", format!("Next Action for {}:", project.name).bold());
    println!("  Phase:         {}", state.current_phase.yellow());
    println!("  Action:        {}", next_action.summary.bold());
    println!("  Command:       {}", next_action.recommended_command.cyan().bold());
    println!("  Assigned Role: {}", next_action.assigned_role.to_string().magenta());
    println!("  Why:           {}\n", next_action.explanation.dimmed());
}

fn handle_phase(root: &Path, action: &PhaseSubcommands) {
    let (_, workflow, mut state) = match load_project_bundle(root) {
        Ok(bundle) => bundle,
        Err(e) => {
            eprintln!("{} Failed loading project: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };

    match action {
        PhaseSubcommands::Status => {
            println!("Current Phase: {}", state.current_phase.bold().yellow());
            if let Some(phase) = find_phase(&workflow, &state.current_phase) {
                println!("Role:          {}", phase.role);
                println!("Requires Lead: {}", phase.requires_human_approval);
                if let Some(ref next) = phase.next_phase {
                    println!("Next Phase:    {}", next);
                }
            }
        }
        PhaseSubcommands::Next | PhaseSubcommands::Complete => {
            let current = state.current_phase.clone();
            if let Some(phase) = find_phase(&workflow, &current) {
                if let Some(ref next_id) = phase.next_phase {
                    // Record completion
                    state.completed_phases.push(CompletedPhase {
                        phase: current.clone(),
                        completed_at: chrono::Utc::now().to_rfc3339(),
                        approved_by: "human".to_string(),
                    });
                    state.current_phase = next_id.clone();
                    state.phase_started_at = chrono::Utc::now().to_rfc3339();

                    if let Err(e) = save_state(root, &state) {
                        eprintln!("Failed saving state: {}", e);
                    } else {
                        println!(
                            "{} Phase '{}' completed. Advanced to '{}'.",
                            "✓".green(),
                            current.yellow(),
                            next_id.cyan().bold()
                        );
                    }
                } else {
                    println!("{} Project has reached the final release phase!", "✓".green());
                }
            }
        }
        PhaseSubcommands::Set { phase_id } => {
            if find_phase(&workflow, phase_id).is_some() {
                state.current_phase = phase_id.clone();
                if let Err(e) = save_state(root, &state) {
                    eprintln!("Failed saving state: {}", e);
                } else {
                    println!("{} Phase set to '{}'.", "✓".green(), phase_id.cyan());
                }
            } else {
                eprintln!("{} Phase '{}' not found in workflow.", "Error:".red(), phase_id);
            }
        }
    }
}

fn handle_task(root: &Path, action: Option<TaskSubcommands>) {
    let action = action.unwrap_or(TaskSubcommands::List);

    match action {
        TaskSubcommands::List => match load_tasks(root) {
            Ok(tasks) => {
                if tasks.is_empty() {
                    println!("No tasks defined in .aiflow/tasks.md.");
                    println!("Add a task with '{}'.", "aiflow task add \"Task title\"".cyan());
                } else {
                    println!("\n{}", "Project Tasks:".bold());
                    for t in tasks {
                        let marker = if t.completed {
                            "[x]".green().to_string()
                        } else {
                            "[ ]".yellow().to_string()
                        };
                        println!("  {} {}: {}", marker, t.id.cyan(), t.title);
                    }
                    println!();
                }
            }
            Err(e) => eprintln!("Error loading tasks: {}", e),
        },
        TaskSubcommands::Done { task_id } => match mark_task_done(root, &task_id) {
            Ok(true) => println!("{} Task {} marked as completed.", "✓".green(), task_id.cyan()),
            Ok(false) => println!("{} Task {} not found or already completed.", "!".yellow(), task_id),
            Err(e) => eprintln!("Error updating task: {}", e),
        },
        TaskSubcommands::Add { title } => match add_task(root, &title) {
            Ok(id) => println!("{} Added task {} in .aiflow/tasks.md", "✓".green(), id.cyan()),
            Err(e) => eprintln!("Error adding task: {}", e),
        },
    }
}
