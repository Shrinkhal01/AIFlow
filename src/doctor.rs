use crate::git::inspect_git;
use crate::storage::{is_initialized, load_project_bundle};
use owo_colors::OwoColorize;
use std::path::Path;

#[allow(dead_code)]
pub struct DoctorReport {
    pub checks_passed: usize,
    pub warnings: usize,
    pub errors: usize,
}

pub fn run_doctor(repo_root: &Path) -> DoctorReport {
    let mut passed = 0;
    let mut warnings = 0;
    let mut errors = 0;

    println!("\n{}", "AIFlow Health & Consistency Doctor".bold());
    println!("{}", "====================================".dimmed());

    // 1. Initialization check
    if !is_initialized(repo_root) {
        println!("  {} .aiflow directory not found. Run 'aiflow init'.", "[✗]".red());
        return DoctorReport {
            checks_passed: 0,
            warnings: 0,
            errors: 1,
        };
    } else {
        println!("  {} .aiflow initialized", "[✓]".green());
        passed += 1;
    }

    // 2. Bundle validity
    match load_project_bundle(repo_root) {
        Ok((project, workflow, state)) => {
            println!(
                "  {} Configuration valid (Project: {}, Preset: {}, Phase: {})",
                "[✓]".green(),
                project.name.cyan(),
                workflow.preset.magenta(),
                state.current_phase.yellow()
            );
            passed += 1;

            // Check required artifacts for current phase
            if let Some(phase) = workflow.phases.iter().find(|p| p.id == state.current_phase) {
                for artifact in &phase.required_artifacts {
                    let artifact_path = repo_root.join(artifact);
                    if artifact_path.exists() {
                        println!("  {} Required phase artifact present: {}", "[✓]".green(), artifact);
                        passed += 1;
                    } else {
                        println!(
                            "  {} Missing required artifact for phase '{}': {}",
                            "[!]".yellow(),
                            phase.name,
                            artifact
                        );
                        warnings += 1;
                    }
                }
            }
        }
        Err(e) => {
            println!("  {} Failed loading .aiflow bundle: {}", "[✗]".red(), e);
            errors += 1;
        }
    }

    // 3. Git working tree check
    match inspect_git(repo_root) {
        Ok(git) => {
            println!(
                "  {} Git working tree inspected (Branch: {}, Modified: {}, Untracked: {})",
                "[✓]".green(),
                git.branch.cyan(),
                git.modified_count,
                git.untracked_count
            );
            passed += 1;

            if git.modified_count > 5 {
                println!(
                    "  {} High number of uncommitted files ({}) in working tree",
                    "[!]".yellow(),
                    git.modified_count
                );
                warnings += 1;
            }
        }
        Err(e) => {
            println!("  {} Git inspection failed: {}", "[!]".yellow(), e);
            warnings += 1;
        }
    }

    // 4. Cache gitignore check
    let gitignore = repo_root.join(".gitignore");
    if gitignore.exists() {
        if let Ok(content) = std::fs::read_to_string(&gitignore) {
            if content.contains(".aiflow/.cache/") {
                println!("  {} .aiflow/.cache/ is properly gitignored", "[✓]".green());
                passed += 1;
            } else {
                println!("  {} .aiflow/.cache/ is NOT ignored in .gitignore", "[!]".yellow());
                warnings += 1;
            }
        }
    }

    println!("\n{}", "Summary:".bold());
    println!(
        "  {} passed, {} warnings, {} errors\n",
        passed.to_string().green(),
        warnings.to_string().yellow(),
        errors.to_string().red()
    );

    DoctorReport {
        checks_passed: passed,
        warnings,
        errors,
    }
}
