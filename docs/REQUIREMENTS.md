# Technical Requirements Specification: AIFlow

**Author:** Lead Software Architect & Product Engineer  
**Date:** October 2026  
**Status:** Approved for Architectural Review  
**Project:** AIFlow  

---

## 1. Functional Requirements (Deconstructed)

### FR-01: Repository Discovery & Registry Management
- **FR-01.1 (Directory Traversal):** The system shall scan configured workspace roots (e.g. `~/Projects`, `~/Code`) recursively to identify local Git repositories (directories containing `.git/`).
- **FR-01.2 (Pruning / Ignore Rules):** The discovery engine must strictly ignore vendor, package, and build directories (e.g., `node_modules`, `.venv`, `target`, `dist`, `vendor`, `.cargo`) to maintain high scanning performance.
- **FR-01.3 (Manual Registration):** The system shall provide CLI commands (`aiflow register [path]` and `aiflow unregister [path]`) allowing developers to explicitly track or untrack repositories.
- **FR-01.4 (Persistent Registry):** Tracked repositories shall be stored in a centralized, user-level registry file at `~/.config/aiflow/registry.yaml` or `~/.cache/aiflow/registry.json`.
- **FR-01.5 (Non-Invasive Inspection):** Scanning must operate strictly in read-only mode and must never mutate repository state or touch Git indexes.

### FR-02: Project State Tracking
- **FR-02.1 (Metadata Extraction):** For each tracked project, the system shall track:
  - Project name (from `.aiflow/project.yaml`, directory name, or package manifest).
  - Absolute filesystem path.
  - Active Git branch name.
  - Working tree status (clean vs. modified/untracked files count).
  - Last activity timestamp (latest commit or file modification).
  - Current workflow phase and completion status.
  - Active task and pending tasks count.
  - Blocking issues or unmet preconditions.
  - Test suite status (Passed / Failed / Stale / Unrun).
  - Documentation freshness status (Synchronized / Stale / Missing).
- **FR-02.2 (Multi-Project Aggregation):** The system shall aggregate this state into a consolidated multi-project overview (`aiflow projects`).

### FR-03: Automatic State & Drift Detection Engine
- **FR-03.1 (Heuristic Derivation):** The state engine shall evaluate heuristic signals across multiple layers:
  1. *Git Signals:* Branch naming patterns (e.g. `phase/03-architecture`, `feat/auth`), active unstaged/staged diffs, recent commit messages matching phase conventions (`arch:`, `feat:`, `test:`, `docs:`).
  2. *Artifact Signals:* Presence and completeness of `.aiflow/` markdown specifications (`requirements.md`, `architecture.md`, `tasks.md`).
  3. *Test Signals:* Status of test runs (e.g., `.pytest_cache`, `target/test-reports`, or explicit `aiflow test` invocations).
  4. *Doc Signals:* Comparison of source code modification timestamps vs. documentation modification timestamps.
- **FR-03.2 (Reconciliation & Divergence Detection):** If the declared state in `.aiflow/state.yaml` contradicts the physical repository state (e.g., state says `Phase: Implementation`, but all tasks in `tasks.md` are checked and tests are failing), the system shall highlight the divergence as an alert.
- **FR-03.3 (Manual Override):** Developers can explicitly transition phases (`aiflow phase set <phase>` or `aiflow phase complete`) when automatic heuristics are ambiguous.

### FR-04: Project Artifact Architecture (`.aiflow/`)
- **FR-04.1 (Directory Structure):** A project initialized with AIFlow shall store its configuration and workflow artifacts in a local `.aiflow/` directory.
- **FR-04.2 (Artifact Evaluation & Minimal Duplication):**
  To avoid merge conflicts, clutter, and redundant files, AIFlow establishes a clean distinction between **Durable Version-Controlled Artifacts** and **Transient Local State**:

```
.aiflow/
├── project.yaml          # Project identity, language, test runner, role bindings (Committed)
├── workflow.yaml         # Project-specific phase state machine & transition rules (Committed)
├── requirements.md       # Problem definition, scope, user stories (Committed)
├── architecture.md       # Technical design, data models, ADRs (Committed)
├── tasks.md              # Checkable task list with phase tags (Committed)
├── state.yaml            # Durable phase progress & approvals (Committed)
└── .cache/               # Local test results, scan cache, prompt history (Git-ignored)
```

- **FR-04.3 (No Redundant Decoupled Docs):** Decisions are embedded as an Architecture Decision Records (ADR) section within `architecture.md` (or an optional `decisions/` folder for large systems) rather than forcing separate sparse files for small projects.
- **FR-04.4 (Validation):** The system shall validate YAML schema correctness and ensure markdown headers adhere to expected specifications.

### FR-05: Command-Line Interface (CLI)
- **FR-05.1 (Core Command Set):**
  - `aiflow init`: Initialize `.aiflow/` in current repository with intelligent project type auto-detection.
  - `aiflow status`: Display detailed, colorized status dashboard for the current project.
  - `aiflow next`: Compute and display the next logical action, recommended commands, and assigned AI role.
  - `aiflow projects`: Display multi-project dashboard across all registered repositories.
  - `aiflow task [list | add | done | active]`: Inspect and manage task checklist in `tasks.md`.
  - `aiflow phase [next | complete | set]`: Advance or inspect workflow phase transitions.
  - `aiflow test [run | status]`: Execute or inspect the project test suite using detected runner.
  - `aiflow docs [check | update]`: Audit documentation staleness or scaffold doc updates.
  - `aiflow doctor`: Health check verifying repository consistency, missing artifacts, uncommitted changes, and workflow compliance.
  - `aiflow prompt --role <role>`: Generate a structured, context-rich prompt scaffold pre-loaded with current spec, tasks, and diffs for the assigned AI agent.

### FR-06: Status & Visual Ergonomics
- **FR-06.1 (Terminal Formatting):** Output shall use modern terminal styling (ANSI color palettes, clean Unicode symbols `✓`, `→`, `○`, `✗`, formatted tables, and collapsible sections).
- **FR-06.2 (Information Hierarchy):** The `aiflow status` command must present information in strict order of developer cognitive priority:
  1. Project Identity & Path
  2. Phased Workflow Progress Tracker (visual breadcrumb)
  3. Git State (branch, uncommitted changes, ahead/behind)
  4. Test State (passed, failed, unrun)
  5. Active Task
  6. Documentation Freshness
  7. **Next Logical Action & Assigned Role/Agent**

### FR-07: Git Integration & Non-Destructive Invariance
- **FR-07.1 (Safety Guarantee):** AIFlow shall **never** perform destructive Git operations (e.g. no automatic `git reset --hard`, no automatic branch deletion, no unapproved stash drops).
- **FR-07.2 (Inspection Capabilities):** Reads:
  - Current HEAD commit SHA and message.
  - Current branch name.
  - Working tree dirty status (`git status --porcelain=v1`).
  - Recent commit history (last 10 commits).
  - Changed files between current branch and base branch (`main`/`master`).

### FR-08: Test Runner Abstraction Layer
- **FR-08.1 (Ecosystem Auto-Detection):** Automatically recognize:
  - Python: `pytest`, `unittest` (detects `pyproject.toml`, `pytest.ini`, `setup.cfg`).
  - Rust: `cargo test` (detects `Cargo.toml`).
  - Node/TypeScript: `npm test`, `pnpm test`, `yarn test`, `vitest`, `jest` (detects `package.json`).
  - Go: `go test ./...` (detects `go.mod`).
  - Java/Kotlin: `mvn test`, `gradle test` (detects `pom.xml`, `build.gradle`).
- **FR-08.2 (Exit Code & Report Parsing):** Capture exit codes, test count, and failure summaries without interfering with standard test execution output.

### FR-09: Documentation Staleness Engine
- **FR-09.1 (Staleness Heuristic):** AIFlow evaluates the commit history of documentation files (`README.md`, `docs/*`) against source code files (`src/*`, `lib/*`, `pkg/*`).
- **FR-09.2 (Alert Threshold):** If significant source changes (e.g., >5 commits or >100 lines changed) have occurred since the last documentation touch, flag documentation as `Stale` in `aiflow status`.

### FR-10: Vendor-Agnostic Role & Provider Mapping
- **FR-10.1 (Role Abstraction):** The workflow engine references abstract roles: `architect`, `implementer`, `reviewer`, `tester`, `approver`.
- **FR-10.2 (Configurable Provider Mappings):** Users configure providers in `~/.config/aiflow/config.yaml` or `.aiflow/project.yaml`:
  ```yaml
  roles:
    architect: claude-3-7-sonnet
    implementer: codex / cursor
    reviewer: gemini-2.5-pro
    tester: gemini-2.5-pro
    approver: human
  ```

---

## 2. Non-Functional Requirements (NFR)

| ID | Category | Requirement | Target Metric |
| :--- | :--- | :--- | :--- |
| **NFR-01** | **Performance** | `aiflow status` latency | $\le 30\text{ ms}$ on standard repositories |
| **NFR-02** | **Performance** | `aiflow projects` multi-repo scan latency | $\le 200\text{ ms}$ for 50 repositories |
| **NFR-03** | **Distribution** | Zero runtime dependencies | Standalone static binary (no required Python/Node/JVM) |
| **NFR-04** | **Platform** | Operating System Support | macOS (Apple Silicon + Intel x86_64) primary; Linux secondary |
| **NFR-05** | **Privacy** | Local-First Architecture | Zero telemetry; no source code or metadata transmitted to external clouds |
| **NFR-06** | **Reliability** | Git Safety | 0% destructive Git commands executed automatically |
| **NFR-07** | **Maintainability** | Solo Developer Ergonomics | Clean modular separation, strict type safety, unit test coverage $\ge 80\%$ |

---

## 3. Analysis of Ambiguous Requirements & Architectural Decisions

### Ambiguity 1: Heuristic State Detection vs. Declared Canonical State
- **Problem:** If automatic heuristics deduce that the project is in `Testing` (because source files changed), but the developer is still writing implementation code in `Implementation`, which state takes precedence?
- **Decision:** **Declared state in `.aiflow/state.yaml` is the canonical state; heuristics provide diagnostic alerts and suggested transitions.**
  - `aiflow status` displays: `Current Phase: Implementation (Declared)`.
  - If heuristics observe that all tasks in `tasks.md` are marked complete, AIFlow outputs a recommendation: *"All tasks complete. Ready to transition to Testing (`aiflow phase next`)."*
  - The developer is never forcibly bumped to the next phase without confirmation.

### Ambiguity 2: Handling Branch Divergence and Merge Conflicts in `.aiflow/`
- **Problem:** If state is stored in `.aiflow/state.yaml` and developers switch Git branches, will switching branches cause merge conflicts or false state regressions?
- **Decision:** **Two-tiered state architecture:**
  1. *Project Workflow Specification (`workflow.yaml`, `requirements.md`, `architecture.md`):* Branch-tracked in Git.
  2. *Branch-Specific State (`.aiflow/state.yaml`):* Contains the phase progress of the *current branch*. When branching from `main` to `feature/foo`, the feature branch inherits `main`'s approved architecture and requirements, while tracking its own task and implementation progress.
  3. *Volatile Test & Scan Caches:* Strictly stored in `.aiflow/.cache/` (added to `.gitignore` during `aiflow init`).

### Ambiguity 3: Passive Status Inspection vs. Active Test Execution
- **Problem:** If `aiflow status` runs the full test suite every time the developer types `aiflow status`, the command will take 10+ seconds, destroying CLI ergonomics.
- **Decision:** **`aiflow status` is strictly passive.** It inspects test run markers, timestamp caches, or previous exit code records in `.aiflow/.cache/test_results.json`. The user explicitly runs `aiflow test` or `aiflow doctor` to execute the actual test suite.

### Ambiguity 4: AI Agent Invocation (Direct API vs. Scaffolding/Prompt Generation)
- **Problem:** Should AIFlow call LLM APIs (requiring API keys, token metering, billing management, and managing streaming API errors)?
- **Decision:** **AIFlow v1 prioritizes prompt and context scaffolding (`aiflow prompt --role <role>`) and CLI invocation delegation.**
  - It generates formatted markdown prompts containing the precise specification context, task definitions, and Git diffs ready for the assigned agent (Claude Code, Cursor, Codex, Gemini).
  - Direct API calling is architected behind a modular `ProviderAdapter` trait for future expansion without cluttering v1.

---

## 4. Technical Risks & Mitigations

| Risk ID | Risk Description | Severity | Mitigation Strategy |
| :--- | :--- | :---: | :--- |
| **TR-01** | **Filesystem Traversal Performance Degradation:** Deep directory trees (`node_modules`, build artifacts) can freeze discovery. | High | Use the `ignore` library (used by `ripgrep`) to automatically honor `.gitignore` and default skip patterns (`vendor`, `target`, `node_modules`). |
| **TR-02** | **Git Index Lock Contention:** Reading Git state while an IDE or agent is committing might cause index lock collisions. | Medium | Use read-only Git operations (direct filesystem tree inspection or `git status --porcelain` with fallback handling) and never acquire exclusive index locks. |
| **TR-03** | **Heterogeneous Test Runners:** Non-standard test frameworks may fail auto-detection. | Medium | Provide explicit override in `.aiflow/project.yaml` (`test_command: "make test"`) when auto-detection cannot resolve the test runner. |
| **TR-04** | **Doc Staleness False Positives:** Trivial source updates (typos, comments) might incorrectly mark documentation as stale. | Low | Configure line-change thresholds and commit message ignore filters (`chore:`, `style:`) in staleness calculation. |
