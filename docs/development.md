# Development

How to build, test, run, drive, profile, and debug the IDE. Command names are the **intended
contract**; treat anything unverified as `Unknown` and record what you learn in
[Verified facts](#verified-facts) rather than in prose.

## Prerequisites

| Requirement | Why | How to verify |
|---|---|---|
| Rust toolchain | Builds the IDE | `rustc --version`, `cargo --version`; if `rust-toolchain.toml` exists, `rustup show` |
| Cargo | Build and test driver | included with the toolchain |
| Native windowing/graphics libraries | egui backend on Linux/BSD | distribution packages for the chosen backend (`winit`/`wgpu`/`glow`) |
| C++ build tools (Windows) | Linking the native backend through the MSVC toolchain | `rustup show`; "Desktop development with C++" in Visual Studio Build Tools |
| Xcode Command Line Tools (macOS) | Compiler, linker, and system headers for the native backend | `xcode-select -p` |
| A JDK | Only for features that compile, run, or analyse Java code | `java -version`, `javac -version` |
| A build tool | Only for project-level build integration | `gradle --version` or `./mvnw -v` inside a test project |

Pin the toolchain in `rust-toolchain.toml` when the project starts building; an unpinned toolchain
makes "works on my machine" the default. Record the MSRV in [Verified facts](#verified-facts) when
one is chosen.

On Windows, the same commands work in PowerShell; use `$env:RUST_BACKTRACE = "1"` instead of the
POSIX `VAR=1 command` form.

## Build and run

From the repository root:

```sh
cargo check                 # fast type-level feedback
cargo build                 # debug build
cargo build --release       # optimised build (needed for realistic UI performance)
cargo run -p kaffein        # desktop binary (verified 2026-10-26)
```

Rules of thumb:

- `cargo check` is the inner-loop command; run it before every commit.
- UI performance work needs `--release`; a debug build misleads by an order of magnitude.
- If the workspace splits the engine from the binary, run the binary package explicitly rather than
  relying on `cargo run` picking a default member.

## Test and quality gates

```sh
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run the gates the project actually supports and report failures verbatim. A gate that cannot run on
the current platform is recorded as not run, not as passing.

### Test taxonomy

An IDE needs more than unit tests. Each level answers a different question; keep the expensive
levels small and deterministic.

| Level | Question it answers | Shape | Runtime |
|---|---|---|---|
| **Unit** | Does the domain rule hold? | Pure functions and values, no window, no disk | milliseconds |
| **Command** | Does the action do what its identifier promises? | Build a context, dispatch a command, assert state and result | milliseconds |
| **Document lifecycle** | Do open/edit/undo/save/reload transitions compose? | Real temporary files, scripted transitions | tens of milliseconds |
| **Golden / snapshot** | Did rendering or formatting change unexpectedly? | Stable input, committed expected output, diff on failure | fast, but review diffs |
| **Oracle** | Does our output match the reference tool? | Run `javac`/build tool on the same input; compare diagnostics or artifacts | seconds |
| **UI smoke** | Does the app assemble and survive a frame? | Headless-ish egui context, drive one frame | seconds |
| **Performance budget** | Are we inside the frame/interaction budget? | Timed scenario on a named machine, recorded p50/p95 | seconds, manual |

Guidelines:

- A bug fix lands with a test at the **lowest** level that can reproduce it.
- Command tests are the backbone: they cover behaviour without pixels and survive UI rewrites.
- Golden tests must state what a mismatch means and who should approve an update.
- Oracle tests must pin the reference tool version; "matches javac" is meaningless without it.
- Never make a test depend on network access, a developer's home directory, or an installed Gradle
  cache. Fixtures are created in temporary directories and deleted.

### Fixtures and corpora

- Commit small fixtures; generate large ones.
- If an external corpus is used (real-world Java projects, malformed sources, archives), pin it by
  version and checksum and document the fetch command. A corpus that silently updates converts
  regressions into noise.
- Keep one fixture per behaviour under test, named for the behaviour, not for the reporter.

## Drive the app programmatically

Prefer testing and automating through commands and services rather than through mouse and keyboard.
Egui has no built-in end-to-end automation harness; assume nothing and document what is actually
proven.

Use this order of preference:

1. **In-process command execution.** Construct the smallest valid application context, dispatch a
   command with explicit arguments, assert the result and the resulting state. This is the default
   for tests.
2. **Headless CLI.** A separate binary that loads a project, runs an action list, and writes output.
   Makes regressions reproducible in CI and enables batch verification.
3. **Control channel.** A loopback-only, authenticated endpoint for inspecting UI state, injecting
   input, and capturing offscreen screenshots. Useful for visual evidence; dangerous if it ships
   enabled by default — see [`security.md`](security.md) › Automation.
4. **UI automation.** Last resort, for focus, drag, and layout behaviour that has no command-level
   expression. Keep it small and platform-scoped.

Every one of these must call the same commands as the UI. If an automation path exists that the UI
does not use, that is a defect in the command model.

### Capturing evidence

Documentation screenshots and parity evidence should be reproducible, not hand-cropped:

- Render at a fixed window size, scale factor, and theme.
- Name the file after the behaviour (`project-tree-compact.png`), not the date.
- Record the command or scenario that produced it next to the image reference.
- Refresh images when the design tokens change; a stale screenshot is a false status claim.

### Automation interface status

**`Unknown`.** Before documenting an interface as available, find it in the code and prove it works:
a CLI help output, a test that starts the server, or a captured response. Do not add a production
control endpoint only for tests without a security review.

## Performance

Performance claims are measurements with a protocol, or they are opinions. When a change targets
performance, record:

| Field | Example |
|---|---|
| Scenario | "open 5,000-file Gradle project, warm caches" |
| Machine | CPU, core count, RAM, GPU, OS build |
| Build | `--release`, optimisation flags, feature set |
| Metric | frame p50/p95, time to first paint, index completion, command latency |
| Before / after | both re-measured on the same machine, same session |
| Budget | the threshold that makes the scenario pass or fail |

Rules:

- Measure on a named machine; "faster" without hardware is not a result.
- Report p50 and p95. Averages hide the freezes users feel.
- Separate computation time from UI frame latency; they fail differently.
- Keep a small, stable set of budgeted scenarios. Every scenario you add costs maintenance.
- Regressions in a budgeted scenario block the change unless the budget is explicitly revised.

## Profiling

Profiling answers "where did the time go"; the numbers in [Performance](#performance) only say that
it went somewhere. The tool is `Unknown` until a profiler is wired into the build, so record the
tool, its version, the command that starts it, and a capture before citing a profile as evidence.

| Question | Approach | Status |
|---|---|---|
| Is filesystem, process, or network work running on the update thread? | Log entry/exit around the update and the suspected boundary; confirm the work belongs in a job | `Proposed` |
| Where does CPU time go inside one scenario? | Sample a release build while replaying a budgeted scenario; keep the capture with the measurement | `Unknown` |
| Is memory or a resource growing over a session? | Repeat the same scenario and sample process memory at fixed intervals | `Unknown` |
| Which syscall or allocation dominates? | Follow up with an OS-level profiler once the scenario is reproducible | `Unknown` |

Rules:

- Profile a `--release` build; a debug profile is not evidence for a shipped build.
- Profile the named scenario and machine from [Performance](#performance), not an ad-hoc session.
- Keep the capture (flamegraph, trace, or log) next to the before/after numbers, with commit and date.
- Keep profiling instrumentation behind an opt-in feature; it does not belong in release artifacts.
- A profile that cannot be reproduced is an anecdote; label it as such.

## Debugging

| Symptom | First move |
|---|---|
| Panic or abort | `RUST_BACKTRACE=1`, then minimise to a command test |
| UI freezes or stutters | check for filesystem/process work on the update thread; profile the frame |
| Layout/input misbehaviour | reduce to one panel or widget; check scale factor and min/max sizes |
| Wrong state after an action | assert the command result and the state transition, not the rendered output |
| Compiler/build output missing | inspect the process invocation: arguments, working directory, environment, exit status |
| Works locally, fails in CI | compare toolchain, JDK, locale, and line endings; check for path assumptions |
| Intermittent failure | look for ordering assumptions, timeouts, or shared temp paths before adding retries |

Practical notes:

- Log at boundaries (command entry/exit, process start/exit, file transitions), not inside tight
  loops.
- Include the identifiers in log lines: command id, job id, document id. Timestamps alone are not
  debuggable.
- Use `RUST_LOG` only if a compatible subscriber is configured; otherwise `eprintln!` during
  investigation and remove it before review.
- Never log secrets, tokens, full environment dumps, or user source code.

## Verified facts

Fill each row from the repository, with a date and the command that produced it. Until then the row
is `Unknown` — do not guess.

| Fact | Value | Status | Evidence | Date |
|---|---|---|---|---|
| Workspace members | `kaffein`, `kaffein-app`, `kaffein-domain`, `kaffein-infra`, `kaffein-ui` | `Implemented` | `cargo metadata --no-deps --format-version 1` | 2026-10-26 |
| Main binary / package | `kaffein`, target `crates/kaffein/src/main.rs` | `Implemented` | `cargo metadata` targets | 2026-10-26 |
| Rust toolchain / MSRV | Pinned `1.98.1`; MSRV declared `1.85` but only `1.98.1` has built the workspace | `Partial` | `rust-toolchain.toml`, `[workspace.package] rust-version` | 2026-10-26 |
| Supported operating systems | Windows (window launched); macOS and Linux unverified | `Partial` | `target\debug\kaffein.exe`, window title `Kaffein` | 2026-10-26 |
| CI gates | None configured | `Unknown` | no `.github/workflows` or other CI configuration in the repository | 2026-10-26 |
| Required environment variables | None | `Implemented` | `crates/kaffein/src/main.rs` constructs services without reading the environment; launch succeeded | 2026-10-26 |
| Logging configuration | No subscriber or log file; command outcomes surface in the status bar | `Partial` | `kaffein_app::Workspace::record_outcome`; gap: persistent logs and `RUST_LOG` integration | 2026-10-26 |
| Automation entry points | None | `Planned` | no CLI, socket listener, or control channel in the workspace | 2026-10-26 |
| JDK discovery order | Not implemented | `Planned` | no JDK module in the workspace | 2026-10-26 |
| GUI backend | `eframe 0.29.1` with `glow` and bundled default fonts | `Implemented` | `Cargo.toml`; `cargo tree --depth 1 -p kaffein` | 2026-10-26 |
| Build, format, lint, test | `cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace` all passed; 39 tests | `Verified` | command output | 2026-10-26 |
| App launch smoke test | Debug binary stayed running for 6 s with window title `Kaffein` and empty stderr | `Verified` | `Start-Process .\target\debug\kaffein.exe` | 2026-10-26 |

## Verification

1. Run `cargo metadata --no-deps --format-version 1` and fill the workspace, binary, and target rows.
2. Read `rust-toolchain.toml` and `Cargo.toml` for the toolchain and MSRV.
3. Read the CI configuration; list each job and the command it runs. Anything not in CI is a local
   gate, and must be labelled as such.
4. Search for a CLI target, socket listener, or test harness; record each one with the file path.
5. Execute one budgeted performance scenario twice and store both results with hardware details.
6. Choose a profiler, record the tool, version, and command, and attach one capture to a budgeted
   scenario; until then the Profiling table stays `Unknown`.
7. Date every row you fill. Undated rows revert to `Unknown`.

