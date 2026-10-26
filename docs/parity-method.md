# Parity method

How this project decides how far along it is. [`ROADMAP.md`](../ROADMAP.md) publishes the numbers;
this document defines what they mean, how they are produced, and how they are audited.

The method exists because IDE parity claims fail in a predictable way: a menu item exists, so the
feature is called "done"; the command opens a dialog, so the workflow is called "complete". Both are
false. Coverage, workflow completion, and quality are three different measurements and must stay
separate.

## Definitions

| Term | Definition |
|---|---|
| **Capability** | A discrete thing a user can do, described in reference-product language ("go to declaration", "rename symbol", "run a JUnit test") |
| **Workflow** | An ordered, end-to-end path a user performs to reach a result, with a start state and an observable end state |
| **Evidence level** | How strongly a capability is known to work (see below) |
| **Scope** | The explicit set of capabilities and workflows counted, with exclusions and reasons |
| **Parity** | Behaviour a user can rely on, measured against the agreed scope — never byte-identical internals |

Three published numbers, computed from the same evidence base:

1. **Feature coverage** — breadth. How much of the in-scope capability surface exists *and* works.
2. **Workflow parity** — depth. How many end-to-end workflows pass their acceptance criteria.
3. **Readiness** — quality. How safe it is to rely on the thing in daily work, weighted by what
   users depend on.

Coverage overstates by design; that is why it is never published alone.

## Scope

Record the scope before counting anything. A number without a scope is not comparable to anything,
including its own past value.

```md
## Scope — <date>

- **Reference product:** IntelliJ IDEA <edition> <version> (<build>)
- **Languages:** Java <versions>
- **Build systems:** <Gradle / Maven versions or "none">
- **Platforms counted:** <OS list>; others are out of scope
- **Capability areas:** <list of areas, e.g. editing, navigation, refactoring, build, run/debug, VCS>
- **Explicitly out of scope:** <items and the decision that excluded them>
- **Evidence cut-off:** <commit / build / date>
```

Scope changes are dated entries, not silent edits. If the reference product version changes, every
number resets until it is re-measured; comparing across versions is not a trend.

## Evidence levels

Assign exactly one level per capability. The level determines the credit used in the coverage
formula.

| Level | Name | What is proven | Minimum artefact | Credit |
|---|---|---|---|---|
| **E0** | Absent | Nothing exists | — | 0.0 |
| **E1** | Visible | A control, menu item, or panel exists | screenshot or path | 0.2 |
| **E2** | Wired | The control dispatches a command that changes state | test name or command output | 0.5 |
| **E3** | Tested | Success and failure paths are covered by automated tests | test names, green run | 0.8 |
| **E4** | Oracle-verified | Output matches the reference tool on a pinned corpus or fixture set | corpus name, matched/total, tool version | 1.0 |
| **E5** | Field-verified | Real users used it on real projects without a blocking defect | issue links, dates, reproduction notes | 1.0 |

Rules:

- A capability with a visible control and no working command is **E1**, and E1 is worth almost
  nothing. This is the single most important rule in this document.
- Partial capabilities record the covered subset and the gap. Credit the subset by weight; never
  round up because "the rest is small".
- E4 and E5 both earn full credit, but they answer different questions. Prefer E4 for algorithmic
  behaviour (parsing, formatting, diagnostics) and E5 for interaction-heavy behaviour.
- Never assign a level from memory. The artefact column is mandatory in the evidence table.
- A capability degrades when a regression is found: re-grade, do not leave the old level in place.

## Scoring

### Feature coverage

```
coverage = Σ (capability_weight × credit(level)) / Σ capability_weight
```

- `capability_weight` defaults to 1. Use weights only for clearly disproportionate capabilities
  (e.g. "build a project" outweighs "toggle a comment"), and record the reason next to the weight.
- Report coverage by area as well as overall; an overall number hides an empty area.
- Report coverage with the evidence-level distribution (`E1: 40, E2: 22, E3: 31, …`). Two projects
  at 70% coverage with different distributions are not comparable.

### Workflow parity

```
workflow_parity = passing_workflows / in_scope_workflows
```

A workflow passes only when **all** of its acceptance criteria hold. Each workflow is defined with:

```md
### <Workflow name>

- **Preconditions:** <project shape, tool versions, settings>
- **Steps:** <numbered, reproducible>
- **Expected result:** <observable, not "it works">
- **Evidence:** <test name / captured output / screenshot path>
- **Platform:** <where it was verified>
- **Last verified:** <date>
```

A workflow that requires a workaround — manual refresh, editing a config file by hand, restarting
the IDE — does not pass. Note the workaround as the gap.

### Readiness

Readiness scores dimensions that breadth cannot see, weighted by how much daily use depends on them.

```md
readiness = ( Σ (dimension_weight × dimension_score) / Σ dimension_weight ) × Π discount
```

- Weights are normalized exactly as in [Feature coverage](#feature-coverage), so the weighted score
  stays in 0–1 before discounts are applied. A weight set that does not sum to 1 is still valid
  input; the division is what keeps the published number comparable across reviews.
- `dimension_score` is 0–1, justified by evidence, not by feel.
- Discounts capture cross-cutting drag (for example interaction latency, crash rate, memory growth).
  Each discount needs its own evidence and a multiplier between 0.8 and 1.0.
- Publish the range, not the point estimate, when a dimension is estimated rather than measured.
- Keep the weights stable across reviews. Changing a weight is a methodology change and belongs in
  the revision history.

## Recording evidence

### Capability table

One row per capability, grouped by area. This is the raw material for every number. The single row
below is an **illustrative shape only**: it is not a measurement of this repository, and the path and
test it names do not exist.

| ID | Capability | Area | Level | Evidence | Gap | Date |
|---|---|---|---|---|---|---|
| `NAV-001` | Go to declaration | navigation | `E2` | `path/to/goto.rs`, `goto_declaration_test` | no cross-module case | `<YYYY-MM-DD>` |

Keep IDs stable across reviews. A deleted capability keeps its row marked removed; reusing an ID for
something else corrupts history.

### Measurement log

Every measurement is an append-only entry. Never rewrite an old one; add a new one and let the
latest supersede it.

```md
### <YYYY-MM-DD> — <what changed>

- **Scope:** <scope version / reference build>
- **Coverage:** <value> (was <value>) — <areas that moved>
- **Workflow parity:** <passed>/<total> (was <x>/<y>)
- **Readiness:** <value> (range) — <discounts applied>
- **Evidence added:** <new tests, corpora, field reports>
- **Still estimated:** <rows that are not measured>
```

## Re-measurement

1. Freeze the evidence cut-off (commit or build) and record it.
2. Re-run the automated measurements first: tests, oracle comparisons, budgeted scenarios.
3. Re-grade only the capabilities whose evidence changed; leave the rest with their dates.
4. Update the workflow list; verify at least the workflows whose area changed.
5. Recompute with the published weights; if a weight must change, update this document in the same
   change and add a revision entry.
6. Publish the revision entry, not just the new number.

Cadence: at every milestone gate, and whenever a measurement invalidates the headline. A number
that is never re-measured is a historical statement, and must be labelled with its date.

## Anti-patterns

| Anti-pattern | Why it is wrong | Do instead |
|---|---|---|
| Menu-item count as parity | Proves wiring, not behaviour | Grade each capability by evidence level |
| "Implemented" from reading the code | The reader is not a test | Cite a test, output, or observation |
| Undated percentages | Cannot be audited or compared | Attach scope and date |
| Averaging measurements with estimates | Hides uncertainty | Publish the range and label estimated rows |
| Counting a plan as partial credit | Plans produce no behaviour | E0 until an artefact exists |
| Weighting by "how hard it was" | Effort is not user value | Weight by user dependence |
| Silent re-scoring | History becomes untrustworthy | Append a revision entry |
| One global number | Hides empty areas and shallow areas | Publish by area, plus distribution |

## Revision history

| Date | Change | Author |
|---|---|---|
| 2026-10-26 | Initial method: three numbers, six evidence levels, append-only measurement log | project docs |
| 2026-10-26 | Normalized the readiness formula like coverage; marked the sample capability row and dates as illustrative | documentation audit |

## Verification

This document defines a method; it becomes enforceable when the first scope is recorded. To
activate it:

1. Write the scope block with the reference product version and capability areas.
2. Create the capability table for one area end to end (about 20 rows) and grade it.
3. Define three workflows with acceptance criteria and verify one of them.
4. Compute the first coverage and workflow numbers from those artefacts only.
5. Publish the revision entry in [`ROADMAP.md`](../ROADMAP.md) and link back here.
