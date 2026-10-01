# AIFlow Development Log & Progress Tracker

This document tracks all design decisions, toolchain configurations, milestones completed, and next actions as AIFlow is constructed.

---

## Current Status Overview
- **Active Phase:** Milestone 1 (MVP Single-Repo Flight Controller) — **RELEASED (v0.1.0)**
- **Release Tag:** `v0.1.0`
- **Language / Toolchain:** Rust 1.98.1 (Apple Silicon `aarch64-apple-darwin`), Cargo, Git 2.55.0
- **Workflow Preset:** 5-Phase Standard (`Specify` → `Plan` → `Build` → `Verify` → `Ship`)
- **Codebase Size:** ~780 lines of clean, synchronous, warning-free Rust.

---

## Chronological Progress Log

### Step 1: Research, Product Specification & Architecture (Oct 1, 2026)
- **Competitive Analysis:** Evaluated GitHub Spec Kit (`specify-cli`), OpenSpec, Aider, Claude Code, `git-bug`, and `Taskwarrior`. Documented in [`docs/RESEARCH.md`](docs/RESEARCH.md).
- **Product Strategy:** Defined problem statement, user personas, role specializations (Claude = Architect, Codex = Implementer, Gemini = Reviewer/Tester, Human = Approver), and scope boundaries in [`docs/PRODUCT.md`](docs/PRODUCT.md).
- **Technical Requirements:** Detailed 13 primary functional and non-functional requirements, resolved state ambiguities, and documented technical risk mitigations in [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md).
- **Architecture & Schemas:** Designed layered architecture, data models for `.aiflow/` (`project.yaml`, `workflow.yaml`, `state.yaml`, `tasks.md`, `spec.md`), and testing strategy in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
- **Public Vision:** Authored public [`README.md`](README.md).

### Step 2: Pragmatic Workflow Optimization & Toolchain Decision
- **Decision:** Simplified workflow from 10 enterprise phases down to the **5-Phase Standard Workflow (`Specify` → `Plan` → `Build` → `Verify` → `Ship`)** as default. Preserves full architectural specification depth without excessive process overhead.
- **Language Selection:** Reconfirmed Rust for native Apple Silicon `<10ms` execution, zero runtime dependencies, and single static binary distribution.
- **Toolchain Installation:** Installed `rustup`, `rustc 1.98.1`, and `cargo` on the macOS host; configured `~/.zshrc` to auto-load `. "$HOME/.cargo/env"`.

### Step 3: Milestone 1 MVP Core Implementation
- **Dependencies (`Cargo.toml`):** Configured minimal, proven crates (`clap` v4 derive, `serde`, `serde_yaml`, `serde_json`, `owo-colors`, `thiserror`, `chrono`).
- **Domain Models (`src/domain.rs`):** Implemented `ProjectConfig`, `WorkflowConfig`, `Phase`, `ProjectState`, `TaskItem`, `Role`, and `GitInfo`.
- **Git Inspector (`src/git.rs`):** Implemented safe, non-destructive Git status, branch detection, and commit log reader.
- **Artifact Engine (`src/storage.rs`):** Implemented `.aiflow/` template generator, YAML state persistence, and Markdown task checklist parser.
- **State Machine & Next Action Engine (`src/fsm.rs`):** Implemented dynamic next-action recommendation algorithm based on current phase and completed tasks.
- **CLI Subcommands (`src/cli.rs` & `src/main.rs`):**
  - `aiflow init`: Scaffolds `.aiflow/` with automatic language and project detection.
  - `aiflow status`: Renders rich colorized breadcrumb progress, git working tree status, task progress, and recommended next action.
  - `aiflow next`: Returns immediate next action, assigned AI agent, and recommended command.
  - `aiflow phase [next | complete | set]`: Manages phase transitions with approval records.
  - `aiflow task [list | done | add]`: Manages Markdown checklists in `.aiflow/tasks.md`.
  - `aiflow doctor`: Audits repository consistency, missing files, and git status.
- **Compilation & Verification:** Built in `1.72s` with **zero compiler warnings**. All subcommands verified against live repository.
- **Global Installation:** Installed release binary to `~/.cargo/bin/aiflow`. Configured `~/.zshrc` so `aiflow` can be executed globally from any terminal.

---

## File Manifest

| File | Purpose | Lines of Code |
| :--- | :--- | :---: |
| [`Cargo.toml`](Cargo.toml) | Cargo workspace manifest & dependencies | ~20 |
| [`src/main.rs`](src/main.rs) | CLI entry point, command dispatch, and terminal formatting | ~240 |
| [`src/cli.rs`](src/cli.rs) | Clap command-line parser & subcommands | ~75 |
| [`src/domain.rs`](src/domain.rs) | Core domain models, state structs, and role definitions | ~170 |
| [`src/git.rs`](src/git.rs) | Read-only Git inspector (branch, dirty tree, commits) | ~80 |
| [`src/storage.rs`](src/storage.rs) | `.aiflow/` filesystem manager & Markdown checklist parser | ~175 |
| [`src/fsm.rs`](src/fsm.rs) | Workflow state machine & Next Action recommendation engine | ~120 |
| [`src/doctor.rs`](src/doctor.rs) | Health audit and consistency linter | ~100 |

---

## Recommended Next Steps (Milestone 2)
1. **Multi-Repository Discovery (`aiflow projects`):**
   - Implement workspace directory scanner using `walkdir`/`ignore` to find all Git repos in `~/Projects` or `~/Shrinkhal-Github`.
   - Render multi-project dashboard table showing phase, health, and pending actions across all repos.
2. **Global Configuration:**
   - Store user workspace roots and role mappings in `~/.config/aiflow/config.yaml`.
3. **Pluggable Test Runner Detection:**
   - Detect `cargo test`, `pytest`, `npm test`, `go test` and report pass/fail in `status`.
