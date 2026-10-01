# AIFlow

[![Status: Architecture & Design Phase](https://img.shields.io/badge/Status-Architecture%20%26%20Design%20Phase-blueviolet?style=for-the-badge)](#project-status)
[![License: MIT / Apache 2.0](https://img.shields.io/badge/License-MIT%20%2F%20Apache%202.0-blue?style=for-the-badge)](#license)
[![Target: macOS & Linux](https://img.shields.io/badge/Platform-macOS%20(Apple%20Silicon)%20%7C%20Linux-black?style=for-the-badge&logo=apple)](#technology-stack)

> **A local developer workflow and project-tracking CLI for software engineers building with AI coding agents.**

---

> [!IMPORTANT]
> **Project Status: Architecture & RFC Review**  
> AIFlow is currently in its initial architectural design and specification phase. The documentation and specifications in `docs/` define the product vision, technical architecture, and implementation roadmap. Application code implementation begins following architectural signoff.

---

## The Vision

In 2026, AI coding agents (Claude Code, OpenAI Codex, Gemini CLI, Cursor, Windsurf, Aider) make generating code faster than ever. However, managing multiple repositories across different AI tools introduces **context fragmentation and process chaos**:

- *What project was I working on before switching contexts?*
- *What development phase is this repository in?*
- *Did the agent write code before architecture was approved?*
- *Are the tests passing? Has documentation fallen out of sync?*
- *What is the next logical action, and which AI model should I prompt for it?*

**AIFlow is NOT another AI coding agent.**  
AIFlow is the **developer's flight control system**—a blazing-fast, local-first CLI that tracks Git repositories, continuously derives project phase and readiness from Git, tests, docs, and artifacts, and guides the developer step-by-step through a disciplined development workflow.

---

## Core Philosophy

### 1. The Developer is the Final Decision Maker
AI agents propose and implement; the human developer steers, approves architecture, reviews code, and holds ultimate release authority.

### 2. Cognitive Role Specialization
Rather than using a single model for every task, AIFlow structures development around specialized roles:

```
                          ┌──────────────────────────┐
                          │     Human Developer      │
                          │   Requirements & Approvals│
                          └─────────────┬────────────┘
                                        │
                         ┌──────────────┴──────────────┐
                         ▼                             ▼
              Claude (Architect)              Codex (Implementer)
              • System Design                 • Feature Implementation
              • Technical Tradeoffs           • Refactoring & Bug Fixes
              • Architecture Review           • Routine Development
                         │                             │
                         └──────────────┬──────────────┘
                                        ▼
                              Gemini (Review & Test)
                              • Adversarial Code Review
                              • Test Suite Generation
                              • Independent Verification
```

*(Note: Roles and provider bindings are fully customizable to prevent vendor lock-in.)*

### 3. Git as the Ground Truth
AIFlow is non-invasive and non-destructive. It reads Git branches (`phase/*`, `feat/*`), uncommitted changes, commit messages, and repository files to automatically infer state without locking your index or rewriting history.

### 4. Zero Cloud Overhead & Instant Startup
Built in Rust, AIFlow executes in `<15ms`, stores zero proprietary code in the cloud, sends no telemetry, and requires zero external runtimes (no Python virtual environments or Node installations needed).

---

## Canonical Phased Workflow

AIFlow guides projects through a 10-phase state machine:

```
Requirements ──> Research ──> Architecture ──> Planning ──> Tasks
                                                             │
Release <── Documentation <── Code Review <── Testing <──────┘
```

| Phase | Core Objective | Primary Role | Default Agent | Human Gate |
| :--- | :--- | :--- | :--- | :---: |
| **1. Requirements** | Define problem, user stories, acceptance criteria | Lead | **Human Developer** | **Required** |
| **2. Research** | Evaluate dependencies, prior art, libraries | Architect | **Claude** | Optional |
| **3. Architecture** | System design, interfaces, data models, ADRs | Architect | **Claude** | **Required** |
| **4. Planning** | Phasing strategy, milestones, risk analysis | Architect / Lead | **Claude + Human** | **Required** |
| **5. Tasks** | Granular, executable task list (`tasks.md`) | Implementer | **Claude / Codex** | Optional |
| **6. Implementation**| Writing feature code and internal refactoring | Implementer | **Codex** | Continuous |
| **7. Testing** | Unit, integration, and regression suites | Tester | **Gemini + Codex** | Automated |
| **8. Code Review** | Adversarial review, security, and lint audits | Reviewer | **Gemini + Claude**| **Required** |
| **9. Documentation** | README updates, user manuals, and API docs | Technical Writer | **Gemini / Claude**| Optional |
| **10. Release** | Tagging, release notes, production build | Release Engineer | **Human Developer** | **Required** |

---

## Planned CLI Experience

### Single-Project Status (`aiflow status`)
```text
Project:      Autonomous Navigation (python)
Location:     /Users/shrinkhals/Projects/autonomous-nav
Last Active:  12 minutes ago (commit 8f3c1b2)

Workflow Progress:
  ✓ 1. Requirements       (Approved by Human)
  ✓ 2. Research           (Completed by Claude)
  ✓ 3. Architecture       (Approved by Human)
  ✓ 4. Planning           (Approved by Human)
  ✓ 5. Tasks              (5 defined, 3 complete)
  → 6. Implementation     [ACTIVE - Codex]
  ○ 7. Testing
  ○ 8. Code Review
  ○ 9. Documentation
  ○ 10. Release

Git Status:
  Branch:       phase/02-object-detection
  Working Tree: 3 modified, 1 untracked
  Upstream:     ahead 1 commit

Test Suite (pytest):
  Status:       14 passed, 2 failed
  Last Run:     45 minutes ago

Current Task:
  TASK-04: Implement object-detection visualization

Documentation:
  Status:       Synchronized (README.md updated 2 days ago)

Next Logical Action:
  → Fix 2 failing tests in test_detector.py with Codex/Gemini
  → Run: aiflow test run
  → Assigned Role: Implementer (Codex)
```

### Next Action Advice (`aiflow next`)
```text
Project: Autonomous Navigation
Current Phase: Implementation

Next Action:
  Implement object-detection visualization in src/detector.py

Recommended Steps:
  1. Generate prompt scaffold:
     $ aiflow prompt --role implementer
  2. Execute unit tests:
     $ aiflow test run
  3. Mark task complete once passing:
     $ aiflow task done TASK-04
```

### Multi-Project Dashboard (`aiflow projects`)
```text
AIFlow Multi-Project Dashboard (4 tracked repositories)

Project                 Phase             Git Status         Tests     Health     Action Needed
------------------------------------------------------------------------------------------------------
Autonomous Navigation   Implementation   phase/02-detection 14/2 fail Attention  Fix 2 failed tests
OnyxBookingSystem       Testing          feat/stripe-v2     All pass  Active     Request code review
OnyxLink                Release          main (clean)       All pass  Done       Ready to tag release
AIFlow                  Architecture     main (clean)       N/A       Active     Approve architecture.md
```

### Repository Health Check (`aiflow doctor`)
```text
AIFlow Health Check: Autonomous Navigation

  [✓] Repository structure: Valid
  [✓] Required artifacts: .aiflow/project.yaml, workflow.yaml present
  [✓] Git working tree: Non-destructive inspection clean
  [!] Test suite: 2 failures detected in test_detector.py
  [✓] Documentation freshness: In sync with recent commits

Doctor Summary: 1 issue requiring attention before advancing to Phase 8 (Code Review).
```

---

## Project Artifacts (`.aiflow/`)

Tracked repositories contain a dedicated `.aiflow/` directory:

```text
.aiflow/
├── project.yaml          # Project identity, language, test runner, role bindings
├── workflow.yaml         # Project-specific phase state machine & transition rules
├── requirements.md       # Problem definition, scope, user stories
├── architecture.md       # Technical design, data models, ADRs
├── tasks.md              # Checkable task list with phase tags
├── state.yaml            # Durable phase progress & human approvals
└── .cache/               # Local test results and volatile cache (Git-ignored)
```

---

## Engineering Specifications

Detailed design documents are maintained in the [`docs/`](docs/) directory:

- [**docs/PRODUCT.md**](docs/PRODUCT.md): Comprehensive product vision, personas, scope boundaries, and core concepts.
- [**docs/ARCHITECTURE.md**](docs/ARCHITECTURE.md): Technology stack justification (Rust), layered modular design, data models, state heuristics, and test harness.
- [**docs/REQUIREMENTS.md**](docs/REQUIREMENTS.md): Functional and non-functional requirements, ambiguity resolutions, and technical risks.
- [**docs/PROJECT_PLAN.md**](docs/PROJECT_PLAN.md): Minimum Viable Product (MVP) scope, phased development roadmap, work breakdown structure (WBS), and quality gates.
- [**docs/RESEARCH.md**](docs/RESEARCH.md): In-depth competitive analysis of GitHub Spec Kit, OpenSpec, AI agent workflows, and project tracking CLIs.

---

## Technology Stack

- **Core Implementation:** **Rust** (2024 edition)
- **CLI Framework:** `clap` (v4 with derive macros)
- **Git Integration:** `git2` (libgit2 bindings) with fallback to `git` CLI
- **Terminal Styling & Tables:** `comfy-table`, `owo-colors`, `indicatif`
- **Filesystem Traversal:** `ignore`, `walkdir` (fastest parallel scanning, honoring `.gitignore`)
- **Data Serialization:** `serde`, `serde_yaml`, `pulldown_cmark`
- **Target Distribution:** Single standalone static binary for macOS (`aarch64` Apple Silicon & `x86_64`) and Linux.

---

## Contributing & Development Roadmap

1. **Phase 0: Architecture & RFC Review** *(Current)*
2. **Phase 1: MVP Core (v0.1.0)** — Single-repo `init`, `status`, `next`, `doctor`, and Git state detection.
3. **Phase 2: Multi-Project Tracking (v0.2.0)** — `aiflow projects` multi-repo dashboard and workspace discovery.
4. **Phase 3: Test Runners (v0.3.0)** — Pluggable test runner adapters (pytest, cargo, npm, go).
5. **Phase 4: Agent Roles & Docs (v0.4.0)** — Prompt scaffolding (`aiflow prompt`) and doc staleness engine.
6. **Phase 5: Public Distribution (v1.0.0)** — Homebrew tap, CI/CD, and binary releases.

To participate in the architectural review, please inspect the documents in [`docs/`](docs/) and provide feedback on the proposed specifications.

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
