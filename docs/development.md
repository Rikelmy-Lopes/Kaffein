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
- `cargo check` and `cargo test` do not relink `target/debug/kaffein.exe`. After a UI change, run
  `cargo build -p kaffein` or `cargo run -p kaffein` before inspecting the window; otherwise the
  old binary hides menu and panel changes.
- UI performance work needs `--release`; a debug build misleads by an order of magnitude.
- If the workspace splits the engine from the binary, run the binary package explicitly rather than
  relying on `cargo run` picking a default member.

### Locales and user settings

Bundled catalogs live in `crates/kaffein-i18n/locales/` and are compiled into the binary. At startup
`FileLocaleSource::discover()` also reads the user configuration directory:

| Platform | Configuration root |
|---|---|
| Windows | `%APPDATA%\Kaffein` |
| macOS | `~/Library/Application Support/Kaffein` |
| Linux/BSD | `$XDG_CONFIG_HOME/kaffein`, else `~/.config/kaffein` |
| Fallback | `.kaffein` in the working directory |
| Override | `KAFFEIN_CONFIG_DIR` replaces the root on every platform |

`locales/<code>.json` adds or overrides a language, and `settings.json` stores the selected locale
under `"locale"` while preserving other keys. Invalid language codes, files over 256 KiB, and
malformed catalogs are ignored; the embedded catalog stays in effect. To add a language, copy
[`en.json`](../crates/kaffein-i18n/locales/en.json), translate the entries, set `meta.name`,
`meta.decimal`, `meta.group`, and `meta.date`, and save it as `<code>.json` in `locales/`. No
recompile is needed, and the language menu in [`ui-design.md`](ui-design.md) › Localisation lists the
new locale on the next launch.

Run `cargo test -p kaffein-i18n` after changing keys or catalogs; the completeness test covers every
bundled locale.

## Running Java from the IDE

`run.execute` (**Run** in the toolbar and **Run** menu, shortcut `F5`) compiles and runs the active
Java buffer without saving it. It requires `javac` and `java` on `PATH`; the adapter does not consult
`JAVA_HOME`. The source is written to a unique directory under the OS temp root, compiled with
`javac -encoding UTF-8`, launched with `java -Dfile.encoding=UTF-8 -cp .`, and the directory is
removed afterwards. Compilation times out after 10 s, execution after 15 s, and each captured stream
is capped at 256 KiB. Output appears in the bottom **Console** tab; the tab is selected automatically
when a run starts.

The startup workspace contains an editable **Scratch.java** document with a runnable
`public class Main`. It has no path on disk: **Save** is refused and closing it discards the buffer.
**File › New Scratch File** creates another one.

The JDK integration has ignored tests because they need a real toolchain:

```sh
cargo test -p kaffein-infra -- --ignored
```

On this machine both tests passed against JDK `1.8.0_401` on 2026-10-28. JDK 8 has no source-file
launch mode, which is why the adapter always writes a temporary file.

## Test and quality gates

```sh
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Run the gates the project actually supports and report failures verbatim. A gate that cannot run on
the current platform is recorded as not run, not as passing.

### File budget check

The module layout and the 300-line budget are rules in [`architecture.md`](architecture.md) ›
Module layout. Measure the current files with:

```powershell
Get-ChildItem crates -Recurse -Filter *.rs |
  ForEach-Object { [pscustomobject]@{ Lines = (Get-Content $_.FullName).Count; File = $_.FullName } } |
  Sort-Object Lines -Descending |
  Select-Object -First 10
```

The command reports; it is not a CI gate yet. A file above the budget is a review finding, and the
measured layout is recorded in [`architecture.md`](architecture.md) › Module layout.

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
| Workspace members | `kaffein`, `kaffein-app`, `kaffein-domain`, `kaffein-i18n`, `kaffein-infra`, `kaffein-ui` | `Implemented` | `cargo metadata --no-deps --format-version 1` | 2026-10-26 |
| Main binary / package | `kaffein`, target `crates/kaffein/src/main.rs` | `Implemented` | `cargo metadata` targets | 2026-10-26 |
| Rust toolchain / MSRV | Pinned `1.98.1`; MSRV declared `1.85` but only `1.98.1` has built the workspace | `Partial` | `rust-toolchain.toml`, `[workspace.package] rust-version` | 2026-10-26 |
| Supported operating systems | Windows (window launched); macOS and Linux unverified | `Partial` | `target\debug\kaffein.exe`, window title `Kaffein` | 2026-10-26 |
| CI gates | None configured | `Unknown` | no `.github/workflows` or other CI configuration in the repository | 2026-10-26 |
| Required environment variables | None required; `KAFFEIN_CONFIG_DIR` optionally overrides the locale/settings directory | `Implemented` | `crates/kaffein-infra/src/locale_files.rs` › `config_root`; launch succeeded without it | 2026-10-26 |
| Logging configuration | No subscriber or log file; command outcomes surface in the status bar | `Partial` | `kaffein_app::Workspace::record_outcome`; gap: persistent logs and `RUST_LOG` integration | 2026-10-26 |
| Automation entry points | None | `Planned` | no CLI, socket listener, or control channel in the workspace | 2026-10-26 |
| JDK discovery order | Not implemented | `Planned` | no JDK module in the workspace | 2026-10-26 |
| GUI backend | `eframe 0.29.1` with `glow` and bundled default fonts | `Implemented` | `Cargo.toml`; `cargo tree --depth 1 -p kaffein` | 2026-10-26 |
| Build, format, lint, test | `cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace` all passed; 39 tests | `Verified` | command output | 2026-10-26 |
| Build, format, lint, test after localisation | `cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace` all passed; 43 tests (20 app, 9 domain, 4 i18n, 5 infra, 5 UI) | `Verified` | command output | 2026-10-26 |
| Build, format, lint, test after locale switching | `cargo check --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` all passed; 59 tests (23 app, 9 domain, 12 i18n, 9 infra, 6 UI) | `Verified` | command output | 2026-10-26 |
| Bundled localisation catalogs | `en` and `pt-BR` in `crates/kaffein-i18n/locales/`; the selected locale persists in `settings.json` | `Implemented` | `cargo test -p kaffein-i18n`; `every_declared_key_resolves_in_every_bundled_locale` | 2026-10-26 |
| Largest Rust source file | `crates/kaffein-app/src/registry/catalog.rs` — 243 lines; budget is 300 | `Verified` | [File budget check](#file-budget-check) | 2026-10-26 |
| Largest Rust source file after localisation | `crates/kaffein-app/src/registry/catalog.rs` — 246 lines; `kaffein-i18n/src/keys.rs` is 159 lines; budget is 300 | `Verified` | [File budget check](#file-budget-check) | 2026-10-26 |
| Largest Rust source file after locale switching | `crates/kaffein-app/src/registry/catalog.rs` — 258 lines; `kaffein-i18n/src/localizer.rs` is 253 lines; budget is 300 | `Verified` | [File budget check](#file-budget-check) | 2026-10-26 |
| App launch smoke test | Debug binary stayed running for 6 s with window title `Kaffein` and empty stderr | `Verified` | `Start-Process .\target\debug\kaffein.exe` | 2026-10-26 |
| Java syntax highlighting | Line-oriented lexer in `kaffein-domain/src/java/` (`java/lexer.rs`, `java/lexer/scan.rs`, line-local semantic classifier in `java/lexer/classify.rs`, token kinds in `java/token.rs`) with an incremental line-hash cache and cached egui galley in `kaffein-ui/src/syntax/` | `Implemented` | `cargo test -p kaffein-domain`; `cargo test -p kaffein-ui --lib syntax`; 2026-10-28 | 2026-10-28 |
| Largest Rust source file after Java highlighting | `crates/kaffein-ui/src/syntax/cache.rs` — 291 lines; `crates/kaffein-domain/src/java/lexer/scan.rs` - 280 lines; budget is 300 | `Verified` | [File budget check](#file-budget-check) | 2026-10-27 |
| Largest Rust source file after semantic highlighting | `crates/kaffein-ui/src/syntax/cache.rs` — 296 lines; `crates/kaffein-domain/src/java/lexer/scan.rs` - 280 lines; budget is 300 | `Verified` | [File budget check](#file-budget-check) | 2026-10-28 |
| JDK used for run tests | `java -version` and `javac -version` both report `1.8.0_401` on Windows | `Verified` | command output | 2026-10-28 |
| Java run adapter | `run.execute` writes the active buffer to a temp directory, compiles with `javac`, launches `java`, caps each stream at 256 KiB, times out after 10 s/15 s, and removes the directory; `cargo test -p kaffein-infra -- --ignored` passed, including a packaged source | `Implemented` | `crates/kaffein-infra/src/java_runner.rs`; `compiles_and_runs_a_source_string`, `compiles_and_runs_a_package_declaration` | 2026-10-28 |
| Build, format, lint, test after scratch/run | `cargo check --workspace --all-targets`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` all passed; 122 tests passed, 2 ignored (28 app, 46 domain, 12 i18n, 11 infra + 2 ignored, 25 UI) | `Verified` | command output | 2026-10-28 |
| App launch smoke test after scratch/run | Debug binary stayed running for 4 s after startup, which dispatches scratch creation; stderr was empty | `Verified` | `Start-Process .\target\debug\kaffein.exe -RedirectStandardError <temp>` | 2026-10-28 |
| Largest Rust source file after scratch/run | `crates/kaffein-ui/src/syntax/cache.rs` — 296 lines; `crates/kaffein-app/src/registry/catalog.rs` — 294 lines; `crates/kaffein-app/src/state.rs` — 269 lines; budget is 300 | `Verified` | [File budget check](#file-budget-check) | 2026-10-28 |

## Verification

1. Run `cargo metadata --no-deps --format-version 1` and fill the workspace, binary, and target rows.
2. Read `rust-toolchain.toml` and `Cargo.toml` for the toolchain and MSRV.
3. Read the CI configuration; list each job and the command it runs. Anything not in CI is a local
   gate, and must be labelled as such.
4. Search for a CLI target, socket listener, or test harness; record each one with the file path.
5. Execute one budgeted performance scenario twice and store both results with hardware details.
6. Choose a profiler, record the tool, version, and command, and attach one capture to a budgeted
   scenario; until then the Profiling table stays `Unknown`.
7. Re-run the [File budget check](#file-budget-check) after a module split and update the
   module-layout table in [`architecture.md`](architecture.md).
8. Date every row you fill. Undated rows revert to `Unknown`.

