# Agent Instructions

This repository contains the documentation for a Java IDE being implemented entirely in Rust with
egui.
IntelliJ IDEA is the reference for user experience and feature parity, not a source of
implementation code.

Read this file first, then the document you are about to change. Each technical document ends with a
**Verification** section that tells you how its placeholders get retired.

## Documentation map

The authoritative map is [`README.md`](README.md) › Documentation map; do not duplicate it here.
Read the document you are about to change, then its **Verification** section.

## Non-negotiable project principles

1. Implement the IDE itself in Rust. Do not add Java, Kotlin, JavaScript, or another language as an
   implementation language for IDE components. Invoking Java tools as subprocesses is fine.
2. Use egui for the graphical interface unless the project explicitly documents an approved
   exception.
3. Model executable application actions as commands. UI entry points dispatch commands rather than
   duplicate operation logic.
4. Keep UI, command execution, domain logic, and infrastructure responsibilities separated, with
   dependencies flowing one way.
5. Follow the existing repository structure and conventions. Inspect the code before proposing or
   creating new abstractions.
6. Prefer small, tested, maintainable changes. Do not claim to have run checks that were not run.
7. Use IntelliJ IDEA as a behavioural and UX reference only. Do not copy its source code, icons,
   fonts, layouts, or other proprietary assets.
8. Treat undocumented repository details as unknown until verified. Update documentation with
   observed facts instead of inventing crate names, metrics, or implemented features.
9. Split code by responsibility. One module per concern, one folder per area, and every source file
   stays within the budget in [`docs/architecture.md`](docs/architecture.md) › Module layout. A file
   that no longer fits is split, not appended to.

## How to write in this repository

### Before writing

1. Decide which single document owns the fact (see **One fact, one home** in [`README.md`](README.md)).
   If the fact has no home, propose one instead of duplicating it.
2. Check whether the fact is already stated elsewhere. If it is, link to it; do not restate it.
3. Establish the evidence: a path, a command, a test name, or a dated measurement. If you cannot,
   write the claim as `Unknown` with the procedure that would settle it.

### While writing

- Use the status vocabulary from [`README.md`](README.md) › Documentation conventions. Words such as
  "done", "works", "supported", and "complete" are only valid with a status and evidence attached.
- Keep the present tense for facts and the conditional for design intent ("a command returns…",
  "the UI should dispatch…").
- Prefer tables over prose when the reader needs to compare or fill in values.
- Put verification steps at the end of the document, under `## Verification`, so placeholders are
  actionable rather than decorative.
- Never present a planned control, panel, or command as visible or functional.

### After writing

Update the tables that your change invalidates:

| If you changed… | Update |
|---|---|
| workspace layout, crates, dependency edges | [`docs/architecture.md`](docs/architecture.md) › Crate map, Dependency layers |
| module layout, file split, or file-size budget | [`docs/architecture.md`](docs/architecture.md) › Module layout |
| command model, result/error semantics, async execution | [`docs/architecture.md`](docs/architecture.md) › Command model, and the checklist in [`docs/contributing.md`](docs/contributing.md) |
| document lifecycle, undo, encoding, external-file handling | [`docs/architecture.md`](docs/architecture.md) › Document model |
| build/test/automation commands or toolchain | [`docs/development.md`](docs/development.md) |
| tokens, layout regions, focus rules, component states | [`docs/ui-design.md`](docs/ui-design.md) |
| scope, stage, or any parity number | [`ROADMAP.md`](ROADMAP.md) and [`docs/parity-method.md`](docs/parity-method.md) |
| process invocation, automation endpoints, file handling | [`docs/security.md`](docs/security.md) |

A change that invalidates a table and does not update it is incomplete, even if the code works.

## Definition of done

The canonical checklist is [`docs/contributing.md`](docs/contributing.md) › Definition of done. It
applies to agent-authored changes unchanged; do not restate it here.

## What not to do

- Do not invent crate names, metrics, command IDs, or platform support.
- Do not add "temporary" side channels that bypass command dispatch.
- Do not widen a layer's dependencies for convenience.
- Do not grow a source file past the budget in [`docs/architecture.md`](docs/architecture.md) ›
  Module layout; split it by responsibility instead.
- Do not add `utils.rs`, `helpers.rs`, or `misc.rs` as a crate-level dumping ground.
- Do not commit secrets, tokens, machine paths, or user data into docs or fixtures.
- Do not mark a parity dimension `Verified` from a checklist of controls; see
  [`docs/parity-method.md`](docs/parity-method.md) › Evidence levels.
- Do not silently rewrite another author's dated measurement. Add a new dated row and keep the old
  one for the record.

## When to ask a human

Stop and ask when a decision changes scope rather than behaviour: adding a language, adopting a
plugin runtime, choosing a licence, publishing a parity number, or removing a documented
non-negotiable. These are product decisions, not implementation details.

## Verification

This file states process rules, not implementation facts, so it has no placeholders to retire. Keep
it accurate by checking that its links still resolve and that it does not restate the sources it
points at — [`README.md`](README.md) › Documentation conventions and Documentation map,
[`docs/contributing.md`](docs/contributing.md) › Definition of done,
[`docs/architecture.md`](docs/architecture.md) › Module layout, and the **After writing** table
above. Re-read it whenever one of those sections changes.
