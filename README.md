# Java IDE in Rust

A desktop IDE for Java development, being implemented entirely in Rust with [egui](https://github.com/emilk/egui).
IntelliJ IDEA is the behavioural and visual reference; it is never a source of code, assets, or
implementation.

| | |
|---|---|
| **IDE implementation language** | Rust — no Java, Kotlin, or JavaScript inside IDE components |
| **GUI framework** | egui |
| **Target development language** | Java |
| **Reference product** | IntelliJ IDEA — UX reference only, clean-room |
| **Repository contents** | Rust workspace and documentation (see [Repository scope](#repository-scope)) |
| **Project status** | foundation shell — `Implemented`, not verified; see [`ROADMAP.md`](ROADMAP.md) |

---

## Repository scope

This repository holds the **Rust workspace** and the **documentation** for the IDE. The workspace
currently contains a foundation shell: a window, menu/toolbar, project tree, editor tabs, an editor with
Java syntax highlighting and context-aware completion, an editable in-memory Java scratch document, a
**Run** command that compiles and executes the active buffer through the JDK, bottom tool window tabs
with a run console, a status bar, design tokens, and a command registry with dispatch and undo.
Completion is backed by an in-process symbol index built from the project and open files. Everything
documented elsewhere remains design intent until a test, a launch trace, or a dated measurement
demonstrates it.

The status rules still apply:

- Claims about code, crates, commands, and toolchains carry a status; anything not demonstrated is
  marked **`Unknown`** or **`Proposed`**.
- Nothing is described as implemented because a control, table, or section exists here.
- Placeholders are explicit and carry the verification procedure that will retire them.

If you are looking for "what does this project actually do today", read
[`ROADMAP.md`](ROADMAP.md) first. It is the only document allowed to summarise status.

## Documentation map

| Document | Answers | Primary audience |
|---|---|---|
| [`README.md`](README.md) | What is this, how are these docs organised, how do I trust them? | everyone |
| [`AGENTS.md`](AGENTS.md) | What are the non-negotiables, and where do I record what I learned? | AI agents, contributors |
| [`docs/architecture.md`](docs/architecture.md) | How is the system layered, how is the code split into modules, what is a command, what owns document state? | implementers, reviewers |
| [`docs/development.md`](docs/development.md) | How do I build, test, run, drive, profile, and debug it? | implementers |
| [`docs/contributing.md`](docs/contributing.md) | What makes a change acceptable, and what is the checklist? | contributors, reviewers |
| [`docs/ui-design.md`](docs/ui-design.md) | What does the interface look like, and which tokens/widgets build it? | UI contributors, designers |
| [`ROADMAP.md`](ROADMAP.md) | How far along is it, stage by stage, and what is next? | maintainers, evaluators |
| [`docs/parity-method.md`](docs/parity-method.md) | How are parity numbers defined, scored, and re-measured? | maintainers |
| [`docs/security.md`](docs/security.md) | What can go wrong when an IDE opens a project and runs tools? | implementers, security reviewers |

## Project commitments

1. **The IDE is Rust.** Java tooling may be invoked as a subprocess; Java is never the language an
   IDE component is written in.
2. **egui is the GUI.** Exceptions require a documented decision, not a preference.
3. **Every user-visible action is a command.** Menus, shortcuts, toolbar buttons, the command
   palette, tests, and automation all dispatch the same implementation. No parallel paths.
4. **Layers stay separated.** UI, command dispatch, domain logic, and infrastructure have one-way
   dependencies.
5. **Undo, focus, and errors are part of the feature.** A command that mutates the document without
   undo, or reports failure silently, is unfinished.
6. **Reference, don't copy.** IntelliJ's behaviour informs design; its code, icons, fonts, and
   layouts do not enter the repository.
7. **Documents state evidence, not intentions.** A status is worth what its evidence is worth.
8. **Small, tested, reversible changes.** Big-bang rewrites and speculative abstractions are
   rejected by default.
9. **Code is split by responsibility.** One module per concern, one folder per area, and every
   source file stays within the budget in [`docs/architecture.md`](docs/architecture.md) ›
   Module layout.

## Documentation conventions

### Status vocabulary

Use exactly these words. They are the only allowed status claims.

| Status | Meaning | Minimum evidence |
|---|---|---|
| `Verified` | Behaviour was observed on a named build, date, and platform | command output, test name, or screenshot path |
| `Implemented` | Code exists, but was not independently re-checked | file path or test name |
| `Partial` | Works for a stated subset; the gap is listed | the subset and the gap |
| `Proposed` | A design decision not yet built | the entry in [`ROADMAP.md`](ROADMAP.md) › Decision log, or the section that specifies it |
| `Planned` | Agreed scope with no design yet | the roadmap milestone |
| `Unknown` | Nobody has checked | the verification procedure |
| `Out of scope` | Deliberately excluded | the reason and the decision that set it |

Never write "done", "works", "supported", or "complete" without one of the entries above
attached.

### Evidence rule

Every factual claim carries either a path (`crates/<crate>/...`), a command (`cargo test ...`), a
test name, or a dated measurement. Undated numbers are not measurements and must not be published.

### One fact, one home

Duplicated facts drift. Each fact lives in exactly one document and is linked elsewhere:

| Fact | Canonical home |
|---|---|
| Crate names, module layout, file-size budget, and dependency edges | [`docs/architecture.md`](docs/architecture.md) |
| Commands, tests, toolchain, automation entry points | [`docs/development.md`](docs/development.md) |
| Command and review conventions | [`docs/contributing.md`](docs/contributing.md) |
| Tokens, typography, layout, widget rules | [`docs/ui-design.md`](docs/ui-design.md) |
| Stage, parity numbers, milestones | [`ROADMAP.md`](ROADMAP.md) |
| Scoring rules and evidence formats | [`docs/parity-method.md`](docs/parity-method.md) |
| Threats and mitigations | [`docs/security.md`](docs/security.md) |

## Reading paths

- **New contributor:** [`docs/contributing.md`](docs/contributing.md) → [`docs/development.md`](docs/development.md) →
  [`docs/architecture.md`](docs/architecture.md) → [`docs/ui-design.md`](docs/ui-design.md).
- **AI agent:** [`AGENTS.md`](AGENTS.md) → the document you are about to change → its
  **Verification** section at the bottom. Then follow the recording rules.
- **Reviewer:** [`docs/contributing.md`](docs/contributing.md) checklists → the diff → the doc table that the
  diff should have updated.
- **Evaluator:** [`ROADMAP.md`](ROADMAP.md) → [`docs/parity-method.md`](docs/parity-method.md) for
  how each number was produced.

## Known unknowns

These are the project facts that must be recorded before this documentation can be called
complete. Each has exactly one destination; do not invent values to fill them.

| Unknown | Where it will be recorded | How to establish it |
|---|---|---|
| Supported platforms and CI gates | [`docs/development.md`](docs/development.md) › Verified facts | CI configuration and release workflow |
| Automation entry points | [`docs/development.md`](docs/development.md) › Drive the app programmatically | search for a CLI, control channel, or test harness |
| Parity target version | [`docs/parity-method.md`](docs/parity-method.md) › Scope | explicit maintainer decision recorded in the scope block |

## Contributing in one minute

Read [`docs/contributing.md`](docs/contributing.md) first. The canonical build, test, and quality-gate
commands live in [`docs/development.md`](docs/development.md) (Build and run; Test and quality gates);
do not restate them here. The short version: keep the layering, dispatch a command instead of
duplicating logic, add tests for behaviour and failure paths, and update the document whose table your
change invalidates. Do not report a check you did not run.

## Verification

There is nothing to measure in this document; it describes how the documentation is organised. It is
verified indirectly: when the [Known unknowns](#known-unknowns) table is empty and every document's
own **Verification** section has been run, this page can be called accurate.
