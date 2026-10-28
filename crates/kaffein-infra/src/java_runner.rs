//! JDK adapter for compiling and running an in-memory Java source buffer.
//!
//! The source is written to a private temporary directory, compiled with
//! `javac`, and launched with `java`. The directory is removed afterwards, so
//! the project is never modified and the user does not have to save first.

#[cfg(test)]
mod tests;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use kaffein_domain::{
    package_name, primary_type_name, JavaRunError, JavaRunOutput, JavaRunStage, JavaRunner,
};

const COMPILE_TIMEOUT: Duration = Duration::from_secs(10);
const RUN_TIMEOUT: Duration = Duration::from_secs(15);
const OUTPUT_LIMIT_BYTES: usize = 256 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(25);

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default)]
pub struct StdJavaRunner;

impl StdJavaRunner {
    pub fn new() -> Self {
        Self
    }
}

impl JavaRunner for StdJavaRunner {
    fn run_source(&self, source: &str) -> Result<JavaRunOutput, JavaRunError> {
        let directory = create_temp_dir()?;
        let result = run_in_directory(&directory, source);
        let _ = std::fs::remove_dir_all(&directory);
        result
    }
}

fn run_in_directory(directory: &Path, source: &str) -> Result<JavaRunOutput, JavaRunError> {
    let type_name = primary_type_name(source).unwrap_or("Main");
    let file_name = format!("{type_name}.java");
    std::fs::write(directory.join(&file_name), source).map_err(|error| {
        JavaRunError::io(format!("{}: {error}", directory.join(&file_name).display()))
    })?;

    let compile = run_process(
        "javac",
        &["-encoding", "UTF-8", "-d", ".", file_name.as_str()],
        directory,
        COMPILE_TIMEOUT,
    )?;

    if compile.timed_out || !compile.success {
        return Ok(JavaRunOutput {
            stage: JavaRunStage::Compile,
            stdout: compile.stdout,
            stderr: compile.stderr,
            exit_code: compile.exit_code,
            timed_out: compile.timed_out,
            stdout_truncated: compile.stdout_truncated,
            stderr_truncated: compile.stderr_truncated,
        });
    }

    let class_name = package_name(source)
        .map(|package| format!("{package}.{type_name}"))
        .unwrap_or_else(|| type_name.to_owned());
    let run = run_process(
        "java",
        &["-Dfile.encoding=UTF-8", "-cp", ".", class_name.as_str()],
        directory,
        RUN_TIMEOUT,
    )?;

    Ok(JavaRunOutput {
        stage: JavaRunStage::Run,
        stdout: run.stdout,
        stderr: run.stderr,
        exit_code: run.exit_code,
        timed_out: run.timed_out,
        stdout_truncated: run.stdout_truncated,
        stderr_truncated: run.stderr_truncated,
    })
}

#[derive(Debug)]
struct ProcessOutcome {
    exit_code: Option<i32>,
    success: bool,
    timed_out: bool,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

fn run_process(
    program: &str,
    args: &[&str],
    directory: &Path,
    timeout: Duration,
) -> Result<ProcessOutcome, JavaRunError> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| JavaRunError::unavailable(format!("{program}: {error}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| JavaRunError::other(format!("{program}: stdout was not captured")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| JavaRunError::other(format!("{program}: stderr was not captured")))?;
    let stdout_reader = std::thread::spawn(move || read_limited(stdout, OUTPUT_LIMIT_BYTES));
    let stderr_reader = std::thread::spawn(move || read_limited(stderr, OUTPUT_LIMIT_BYTES));

    let (exit_code, success, timed_out) = wait_for_exit(&mut child, timeout);
    let (stdout, stdout_truncated) = join_output(stdout_reader);
    let (stderr, stderr_truncated) = join_output(stderr_reader);

    Ok(ProcessOutcome {
        exit_code,
        success,
        timed_out,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> (Option<i32>, bool, bool) {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (status.code(), status.success(), false),
            Ok(None) if started.elapsed() < timeout => std::thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let _ = child.kill();
                let status = child.wait().ok();
                return (status.and_then(|status| status.code()), false, true);
            }
            Err(_) => {
                let _ = child.kill();
                let status = child.wait().ok();
                return (status.and_then(|status| status.code()), false, false);
            }
        }
    }
}

fn join_output(handle: std::thread::JoinHandle<(String, bool)>) -> (String, bool) {
    handle.join().unwrap_or_else(|_| (String::new(), false))
}

fn read_limited(mut reader: impl Read, limit: usize) -> (String, bool) {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                if bytes.len() < limit {
                    let remaining = limit - bytes.len();
                    let take = remaining.min(read);
                    bytes.extend_from_slice(&buffer[..take]);
                    truncated |= take < read;
                } else {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }

    (String::from_utf8_lossy(&bytes).into_owned(), truncated)
}

fn create_temp_dir() -> Result<PathBuf, JavaRunError> {
    #[cfg(unix)]
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    #[cfg(not(unix))]
    let builder = std::fs::DirBuilder::new();

    for _ in 0..32 {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path =
            std::env::temp_dir().join(format!("kaffein-run-{}-{nanos}-{id}", std::process::id()));
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(JavaRunError::io(format!("{}: {error}", path.display())));
            }
        }
    }

    Err(JavaRunError::io(
        "could not allocate a unique temporary directory",
    ))
}
