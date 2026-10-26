# Roadmap

> The one-page status summary. Every number here is either **measured** (with a date, a scope, and an
> artefact) or **`Not measured`**. Nothing is filled in from expectation.
> Scoring rules live in [`docs/parity-method.md`](docs/parity-method.md).

## Current stage

**Stage: Foundation — shell implemented, gate not verified.** The workspace launches a desktop shell
on Windows, the command registry is the only action path for the controls it exposes, and the
foundation tests pass. The Foundation exit gate is still open: there is no CI, no layout persistence,
and no visual or contrast verification against a running build.

Stage names are defined by their exit gates, not by effort spent.

| Stage | Exit gate (all criteria required) |
|---|---|
| **Foundation** | Launches and shows a shell; command registry with dispatch and undo; one document type opens, edits, undoes, and saves; tests and CI run; design tokens implemented in the theme module and mirrored in [`docs/ui-design.md`](docs/ui-design.md); layout documented |
| **Core IDE** | Project open with tree navigation; editor with diagnostics from a parser; multi-tab lifecycle including external changes; settings persist; palette and keymap drive every command |
| **Java workflow** | JDK discovery; project model for at least one build system; compile with diagnostics mapped to files; run a main class; console shows process lifecycle correctly |
| **Productivity** | Search across project, go to declaration, rename with preview, test runner, VCS status — each as commands with acceptance-tested workflows |
| **Parity and polish** | Readiness dimensions at target; interaction budgets met; documented recovery paths; accessibility baseline verified |

A stage is entered when the previous gate is *verified*, not when it is mostly done.

## Headline numbers

The first three numbers are measured, or explicitly `Not measured`; Stage is an assessment, not a
measurement.

| Number | Question | Value | Kind |
|---|---|---|---|
| **Feature coverage** | How much of the in-scope capability surface works? | `Not measured` (0 capabilities graded) | measured, per [`docs/parity-method.md`](docs/parity-method.md) |
| **Workflow parity** | How many end-to-end workflows pass their acceptance criteria? | `Not measured` (0 workflows defined) | measured |
| **Readiness** | How reliably can the in-scope workflows be used for daily work? | `Not measured` | weighted estimate with discounts |
| **Stage** | Which gate is verified? | Foundation — started, not verified | assessed |

Rules for publishing:

- A number appears only with its scope, evidence cut-off, and date.
- Coverage is never published alone; it overstates by design.
- Estimated rows publish a range and are labelled `estimated`.
- A number that moves gets a revision entry; a number that stops moving gets re-measured or marked
  stale.

## Dimensions

Status uses the vocabulary in [`README.md`](README.md): `Verified`, `Implemented`, `Partial`,
`Proposed`, `Planned`, `Unknown`, `Out of scope`.

| Dimension | Status | Evidence level | Evidence / next milestone |
|---|---|---|---|
| Application shell and window management | `Partial` | `E1` | `crates/kaffein/src/main.rs`; window `Kaffein` launched 2026-10-26. Gap: layout persistence and narrow-window checks |
| Command registry, dispatch, and undo | `Implemented` | `E3` | `workspace::tests::undo_and_redo_dispatch_through_the_document`, `file_open_without_a_path_is_an_internal_error`; `cargo test --workspace` 2026-10-26 |
| File/project explorer | `Partial` | `E2` | `project_loader::tests::scans_directories_first_and_skips_ignored_folders`, `workspace::tests::opening_a_project_builds_the_tree`. Gap: no filtering or file operations |
| Editor and document lifecycle | `Partial` | `E3` | `creating_a_file_adds_a_dirty_document_without_writing`, `editing_then_saving_writes_the_document_and_clears_dirty_state`, `closing_a_dirty_document_is_refused`. Gap: reload, external change, discard flow |
| Tabs, split views, tool windows | `Partial` | `E2` | `workspace::tests::activating_an_open_document_switches_the_active_tab`; `draw_tabs`. Gap: split views; bottom tool tabs are placeholders |
| Search and navigation | `Unknown` | `E0` | M2: project-wide search, go to declaration |
| Java/JDK discovery and configuration | `Unknown` | `E0` | M2: discovery order recorded, version mismatch diagnosed |
| Build and compilation workflows | `Unknown` | `E0` | M2: one build system, diagnostics mapped to files |
| Run/debug workflows | `Unknown` | `E0` | M2 run; M3 debug if in scope |
| Diagnostics and error navigation | `Unknown` | `E0` | M2: gutter, underline, problem list, jump-to-source |
| Testing workflows | `Unknown` | `E0` | M3: discover, run, and report JUnit tests |
| Refactoring and code intelligence | `Unknown` | `E0` | M3: at least rename with preview and undo |
| Version control integration | `Unknown` | `E0` | M3: status, diff, commit as commands |
| Themes and visual polish | `Partial` | `E1` | `crates/kaffein-ui/src/theme.rs`; `view.toggle_theme` command. Gap: contrast and screenshots not re-measured |
| Keyboard and accessibility | `Partial` | `E1` | `crates/kaffein-app/src/registry.rs` shortcut table (`Ctrl+N/O/S/Z/Y/Q`) dispatches commands. Gap: no palette, focus ring and scale factors unverified |
| Performance and reliability | `Unknown` | `E0` | Budgeted scenarios defined and measured in [`docs/development.md`](docs/development.md) |
| Settings, persistence, extensibility | `Unknown` | `E0` | Settings schema, migration, and a decision on plugins |
| Security posture | `Proposed` | `E0` | [`docs/security.md`](docs/security.md) controls implemented and audited |

An `E0` with status `Proposed` means "designed on paper, nothing built". A status of `Proposed` is
never credit toward a parity number; only evidence levels earn credit.

## Scope

| Axis | In scope | Out of scope (for now) |
|---|---|---|
| Languages | Java | Kotlin, Scala, Groovy language support — `Out of scope` until a decision records otherwise |
| Build systems | One, chosen by decision | Additional build systems follow the same project model |
| Platforms | Windows, macOS, Linux — to be verified in CI | Mobile, web, remote-hosted editing |
| Distribution | Source build first | Signed installers, auto-update — tracked as a milestone, not assumed |
| Extensibility | Commands and automation | Third-party plugin runtime in another language — conflicts with principle 1 |
| Reference product | IntelliJ IDEA, version recorded per measurement | Copying IntelliJ assets, protocols requiring its binaries, or its file formats |

Changing the language or build-system axis is a product decision, not an implementation detail. It
belongs in the decision log below.

## Milestones

Milestones are ordered by dependency. Each has a "done when" that can be observed, and each ends
with a measurement entry.

### M0 — Foundation

**Done when:** the app launches, shows the documented layout, opens one file type, edits with undo
and save, and the command registry is the only action path. CI runs format, lint, and tests.

### M1 — Project and editor

**Done when:** a project folder opens, the tree navigates it, tabs share one document and one undo
stack, external file changes are detected, and settings persist across restarts.

### M2 — Java workflow

**Done when:** the IDE discovers a JDK, loads a project for one build system, compiles it, maps
diagnostics to source positions, and runs a main class with correct process lifecycle in the console.

### M3 — Productivity

**Done when:** project search, go to declaration, rename with preview, and the test runner each pass
an acceptance-tested workflow; VCS status and diff are read-only and reliable.

### M4 — Parity and polish

**Done when:** readiness dimensions meet their targets, interaction budgets are met on a named
machine, recovery paths (crash, lost connection, corrupt settings) are documented and tested, and the
accessibility baseline is verified.

## Decision log

Decisions that change scope or constraints. Append-only; superseded entries stay with a pointer.

| Date | Decision | Rationale | Status |
|---|---|---|---|
| 2026-10-26 | Documentation uses evidence levels and three separate parity numbers instead of one percentage | A single number hid the difference between "a menu item exists" and "the workflow works" | Adopted |
| 2026-10-26 | Public parity numbers are withheld until a scope and evidence base exist | An unmeasured percentage is a false claim | Adopted |
| `TBD` | First build system (Gradle or Maven) | Determines the project model and the largest cost driver | Open |
| `TBD` | Plugin model (if any) | Constrained by the Rust-only principle | Open |
| `TBD` | Distribution and update channel | Determines signing and security requirements | Open |

## What's next

Ordered by dependency. Each item names the artefact that closes it, so "done" is not a judgement
call. The 2026-10-26 foundation shell turned items 1–4 from design into implementation; they stay
listed because their verification is still missing.

1. **Verify the recorded workspace facts in CI.** Run format, lint, and tests on a platform matrix.
   *Closed by:* dated CI job output and updated rows in [`docs/development.md`](docs/development.md).
2. **Extend the command model.** Add the command palette and automation entry point over the existing
   registry; no second dispatch path. *Closed by:* a command test plus palette and headless entry
   points using `Workspace::dispatch`.
3. **Close the document lifecycle.** Reload, external-change detection, and an explicit discard flow
   for dirty documents. *Closed by:* lifecycle tests and the updated Document model table.
4. **Re-measure the design tokens.** Capture both themes at 100/125/150% and re-check contrast.
   *Closed by:* linked screenshots and dated contrast results in [`docs/ui-design.md`](docs/ui-design.md).
5. **Define the parity scope.** Reference product version, capability areas, three workflows.
   *Closed by:* a scope block and a measurement entry in [`docs/parity-method.md`](docs/parity-method.md).
6. **Implement the first security controls.** Project trust consent and automation hardening.
   *Closed by:* status changes in [`docs/security.md`](docs/security.md).
7. **Verify the M0 gate.** Run the Foundation exit criteria against a real build. *Closed by:* dated
   evidence rows for each criterion and `M0` recorded as verified in this file.

**Last assessed:** 2026-10-26 (foundation shell launched and tests run on Windows; CI and visual
verification remain outstanding).

## Revision history

| Date | Change | Evidence |
|---|---|---|
| 2026-10-26 | Roadmap rebuilt around stage gates, three numbers, evidence levels, decision log | documentation audit |
| 2026-10-26 | "Headline numbers" label corrected (Stage is an assessment, not a measured number); Foundation gate now requires tokens implemented in code; next-step 7 replaced with a verifiable gate check | documentation audit |
| 2026-10-26 | Foundation shell implemented: five-crate workspace, command registry/dispatch/undo, editor with tabs, project tree, design tokens, and status bar. Stage remains Foundation, not verified | `cargo test --workspace` (35 passed) and `kaffein.exe` launch on Windows; see [`docs/development.md`](docs/development.md) › Verified facts |
| 2026-10-26 | Added `file.new` (`Ctrl+N`): a new document is created in memory, stays dirty until its first save, and is refused when the path already exists | `workspace::tests::creating_a_file_adds_a_dirty_document_without_writing`, `saving_a_new_document_writes_it_and_clears_dirty_state`, `creating_a_file_that_already_exists_is_refused` |

