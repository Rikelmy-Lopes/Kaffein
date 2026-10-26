# Contributing

This project is easy to contribute to when a change is small, tested, and honest about what it
proves. Read [`architecture.md`](architecture.md) for the layering and [`development.md`](development.md)
for the commands before writing code.

## Ways to contribute

| Kind of contribution | What it looks like | What reviewers check |
|---|---|---|
| Command or feature | A new command plus its UI entry points and tests | Registry entry, undo, error path, one dispatch path |
| Bug fix | A failing test first, then the smallest fix | Reproduction level, no behaviour drift elsewhere |
| Performance work | A measured scenario, before/after, budget | Named machine, p50/p95, no correctness change |
| Documentation | A verified fact replacing an `Unknown` row | Evidence, dated, single home for the fact |
| Design/UI | A token or component in [`ui-design.md`](ui-design.md) terms | Token usage, focus/keyboard, no fake affordances |
| Security | A threat, control, or hardening test | See [`security.md`](security.md) |

## Core rules

### Clean-room implementation

IntelliJ IDEA is a reference for expected workflows and user experience, not a source of code. Do
not copy proprietary source code, icons, fonts, layout files, translations, or implementation
details. Implement features independently in Rust, and use original or appropriately licensed
assets. If a design is inspired by a specific IntelliJ pattern, describe the *behaviour* you are
reproducing in the design note; do not paste the artefact.

### Rust-only IDE implementation

IDE components are written in Rust. External Java tools may be invoked when Java functionality
requires them, but they are tools the IDE uses — never languages in which IDE components are
implemented. A JavaScript or Python runtime embedded in the UI is a scope decision, not a
convenience.

### Architecture and layering

- Preserve the separation between UI, application/commands, domain, and infrastructure.
- UI code dispatches commands; it does not implement their logic or reach for filesystem handles.
- Reuse existing commands and services before adding new ones.
- No circular dependencies, and no upward dependencies (infrastructure must not know about UI).
- Do not introduce abstractions, dependencies, or modules without a concrete need.
- Keep egui types out of domain APIs.
- User-facing text goes through `kaffein-i18n`; add the key constant and every bundled catalog entry
  in the same change instead of embedding a literal or falling back to English-only text.

### Code organisation

- One module, one responsibility; one folder per feature area. Put the responsibility in the module
  doc comment, not in the file name.
- Keep every Rust source file within the 300-line budget in
  [`architecture.md`](architecture.md) › Module layout, tests included. Split along
  responsibilities; do not cut a file at an arbitrary line.
- `lib.rs` and `mod.rs` wire modules and re-export. Behaviour lives in the module that owns it.
- Unit tests live in `src/<module>/tests.rs`; split them into `src/<module>/tests/` by behaviour
  when they outgrow the budget.
- Do not create crate-level `utils.rs`, `helpers.rs`, or `misc.rs`. A helper belongs next to the
  rule it supports.

### Undo, focus, and errors are not optional

- A command that mutates a document registers undo, or documents why it cannot.
- A command that can fail returns a structured error with enough context for the UI to act.
- Long-running commands report progress and honour cancellation.
- A control that cannot currently act is disabled *and* explains why on hover/status.

### Tests

- Add or update tests for behaviour changes, at the lowest level that can reproduce the behaviour.
- Cover the success path, an invalid-input path, and a meaningful failure path.
- Prefer deterministic tests without network access or installed Java tools; when a test needs a
  JDK, tag it and make the requirement explicit.
- Use temporary directories for filesystem tests and clean up afterwards.
- Run the gates and report exactly what happened.

### Style

- Follow existing formatting, naming, and module conventions.
- Keep files within the module layout and file budget in [`architecture.md`](architecture.md) ›
  Module layout.
- Prefer idiomatic Rust, clear names, small focused functions, and explicit error handling.
- Avoid `unwrap()`/`expect()` on paths where failure is expected; a panic in a command is a defect.
- Avoid unnecessary cloning in per-frame code paths.
- Keep `unsafe` minimal, local, and accompanied by a safety comment.
- Keep changes focused; unrelated cleanups belong in a separate change.

### Commits

Follow the repository's conventions if documented. Otherwise use a concise imperative subject
("Add rename command grouping"), keep the body to *why* rather than *what*, and keep unrelated
changes separate. Do not claim a commit or check exists unless it does.

### Licensing and attribution

Record the project licence and every third-party asset with its source, licence, and modification
note. Fonts, icons, themes, and sample projects are the usual offenders. If a licence is not
recorded, the contribution cannot be accepted — an unlicensed asset is a legal defect, not a
documentation gap.

| Item | Licence | Evidence | Status |
|---|---|---|---|
| Project | GPL-3.0-only | [`LICENSE`](../LICENSE); `[workspace.package] license` in [`Cargo.toml`](../Cargo.toml) | `Verified` (2026-10-26) |
| `egui` / `eframe` 0.29.1 | MIT OR Apache-2.0 | `cargo metadata --format-version 1` | `Verified` (2026-10-26) |
| `epaint_default_fonts` 0.29.1 (bundled UI fonts) | (MIT OR Apache-2.0) AND OFL-1.1 AND LicenseRef-UFL-1.0 | `cargo metadata`; used unmodified through `eframe`'s `default_fonts` feature | `Verified` (2026-10-26) |
| `glow` 0.14.2 | MIT OR Apache-2.0 OR Zlib | `cargo metadata` | `Verified` (2026-10-26) |
| `winit` 0.30.13 | Apache-2.0 | `cargo metadata` | `Verified` (2026-10-26) |
| `serde_json` 1.0.154 | MIT OR Apache-2.0 | `cargo metadata`; direct dependency of `kaffein-i18n` and `kaffein-infra` for JSON catalogs and settings | `Verified` (2026-10-26) |
| `serde` / `serde_core` 1.0.229 | MIT OR Apache-2.0 | `cargo metadata`; JSON serialization support | `Verified` (2026-10-26) |
| `itoa` 1.0.18 | MIT OR Apache-2.0 | `cargo metadata`; `serde_json` transient | `Verified` (2026-10-26) |
| `zmij` 1.0.23 | MIT | `cargo metadata`; `serde_json` transient | `Verified` (2026-10-26) |
| `memchr` 2.8.3 | Unlicense OR MIT | `cargo metadata`; transient | `Verified` (2026-10-26) |

No IntelliJ icons, fonts, layouts, translations, or source files are vendored. The complete
transitive licence set is available from `cargo metadata --format-version 1`; re-run it when a
dependency is added and update this table.

## Checklist: adding a command

1. **Inspect existing patterns.** Find similar commands, the registry entry shape, result types,
   and service dependencies. Copy the convention, not the code.
2. **Define the action.** One responsibility, named `area.verb`. List inputs, preconditions, and
   whether it is undoable.
3. **Choose the layer.** Domain behaviour belongs in the domain; orchestration in the application
   layer; filesystem/process access in infrastructure.
4. **Implement in Rust.** No behaviour hidden in a script, shell command, or build step.
5. **Register it.** Add the identifier, localised title key, category key, default shortcut (if
   any), enablement predicate, and argument schema to the single table in
   `crates/kaffein-app/src/registry/catalog.rs`.
6. **Wire every entry point.** Menu, context menu, toolbar, palette, keybinding — all dispatch the
   same command. Do not add per-widget implementations.
7. **Handle outcomes.** Success, structured no-op, cancellation, and failure; surface each
   deliberately.
8. **Handle responsiveness.** Move slow work to a job; keep the frame loop short.
9. **Handle undo.** Register the inverse operation, or document the exclusion in the command's doc
   comment.
10. **Test it.** Command-level tests for the paths above, plus a lifecycle test if it mutates
    documents.
11. **Update the UI.** Consistent labels, disabled states, and feedback with existing patterns.
12. **Update documentation.** The tables named in [`AGENTS.md`](../AGENTS.md) › After writing.
13. **Run checks.** Format, lint, test; quote failures verbatim.

## Checklist: UI change

- [ ] Uses shared tokens from [`ui-design.md`](ui-design.md); no new hard-coded colours or sizes.
- [ ] All user-facing copy resolves through the localisation catalog; no English literals in widgets.
- [ ] Keyboard reachable, with a visible focus ring and a sensible tab order.
- [ ] Shortcut (if any) is registered as a command, not a local key handler.
- [ ] Disabled, hover, active, focus, and selected states are all defined.
- [ ] Empty, loading, error, and very long content states are handled.
- [ ] Works at 100% and at a fractional scale factor, and in a narrow window.
- [ ] No placeholder control that looks functional but does nothing.
- [ ] Screenshot or visual evidence attached and reproducible.

## Checklist: performance change

- [ ] A named, reproducible scenario exists before the change.
- [ ] Before and after measured on the same machine, in the same session, release build.
- [ ] p50 and p95 reported, plus the budget the scenario must meet.
- [ ] Computation time and UI frame latency reported separately.
- [ ] Correctness is unchanged: tests, and golden/oracle output where applicable.
- [ ] The result is recorded with a date in the relevant document.

## Checklist: documentation change

- [ ] The fact has exactly one home; other documents link to it.
- [ ] Every claim carries a status from the vocabulary in [`README.md`](../README.md).
- [ ] No `TBD` was replaced by a guess; verified rows carry evidence and a date.
- [ ] Tables that the change invalidates were updated.
- [ ] Internal links resolve, including anchors.
- [ ] No proprietary text, asset, or screenshot was copied in.

## Commits and pull requests

- One logical change per commit; one concern per pull request.
- The description states the user-visible effect, the tests that prove it, and any parity row it
  moves (with the evidence).
- Prefer small pull requests that can be reviewed in one sitting. If a change needs a design
  decision, write the decision down first and link it.
- Do not mix formatting churn with behaviour changes; it destroys reviewability and blame.
- If a check could not run, say so and why. Reviewers treat silence as a false claim.

## Definition of done

This is the canonical checklist. [`AGENTS.md`](../AGENTS.md) › Definition of done points here instead
of restating it.

- [ ] The change is implemented in Rust; application actions are dispatched as commands.
- [ ] Menus, shortcuts, toolbar, palette, automation, and tests reach the same command implementation.
- [ ] Document mutation is undoable and redoable, or the exclusion is documented with a reason.
- [ ] Failure paths return structured errors and are surfaced in the UI.
- [ ] Long-running work leaves the UI thread and can be cancelled.
- [ ] Tests cover success, invalid input, and a meaningful failure path, at the appropriate level.
- [ ] New or changed code follows the module layout and stays within the file budget in
  [`architecture.md`](architecture.md) › Module layout.
- [ ] The relevant documentation tables were updated and match the implementation.
- [ ] Relevant checks were run; every check reported as run was actually run, and failures are
  quoted verbatim.
- [ ] No unrelated files, generated artifacts, or secrets were included.
- [ ] Existing repository conventions were followed.

## What we do not merge

- Java, Kotlin, or JavaScript implementations of IDE behaviour.
- A second code path that performs the same action as an existing command.
- A visible control with no behaviour behind it.
- An abstraction introduced "for the future" with no current caller.
- A source file that exceeds the file budget, or a crate-level `utils.rs`/`helpers.rs`/`misc.rs`
  used as a dumping ground.
- A parity claim without evidence, or a status word without a status.
- Copied proprietary code, icons, fonts, or layouts.
- A performance claim without a measurement protocol.

## Reviewing

Reviewers check, in order: does it work; is it in the right layer; is it reachable from every entry
point; is it undoable; is it tested; does it tell the truth in the docs. Style comments come after
correctness. If a change cannot be verified by the reviewer, ask for the command or test that proves
it rather than approving on trust.

## Verification

This document contains no factual claims about the implementation, so it has nothing to retire.
When the project establishes a code review process, a licence, or a CI gate, record them here with a
date and update the checklists to match.
