//! Observable Java-run state shared with a worker thread.

use std::sync::{Arc, Mutex, MutexGuard};

use kaffein_domain::{JavaRunError, JavaRunOutput};

/// Observable state of the last or current Java run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RunState {
    #[default]
    Idle,
    Running {
        id: u64,
        source_name: String,
    },
    Finished {
        id: u64,
        source_name: String,
        output: JavaRunOutput,
    },
    Failed {
        id: u64,
        source_name: String,
        detail: String,
    },
}

impl RunState {
    /// A monotonically increasing identity, or `0` while idle.
    pub fn id(&self) -> u64 {
        match self {
            Self::Idle => 0,
            Self::Running { id, .. } | Self::Finished { id, .. } | Self::Failed { id, .. } => *id,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn source_name(&self) -> Option<&str> {
        match self {
            Self::Idle => None,
            Self::Running { source_name, .. }
            | Self::Finished { source_name, .. }
            | Self::Failed { source_name, .. } => Some(source_name),
        }
    }
}

/// A run owned by one worker thread. Finishing it publishes the result for the
/// UI; a stale job cannot overwrite a newer run because the identity is checked.
#[derive(Debug)]
pub(crate) struct RunJob {
    id: u64,
    source_name: String,
    state: Arc<Mutex<Arc<RunState>>>,
}

impl RunJob {
    pub(crate) fn new(id: u64, source_name: String, state: Arc<Mutex<Arc<RunState>>>) -> Self {
        Self {
            id,
            source_name,
            state,
        }
    }

    pub(crate) fn finish(self, result: Result<JavaRunOutput, JavaRunError>) {
        let mut state = lock_run(&self.state);
        if state.id() != self.id {
            return;
        }
        *state = Arc::new(match result {
            Ok(output) => RunState::Finished {
                id: self.id,
                source_name: self.source_name,
                output,
            },
            Err(error) => RunState::Failed {
                id: self.id,
                source_name: self.source_name,
                detail: error.detail().to_owned(),
            },
        });
    }
}

pub(crate) fn lock_run(state: &Arc<Mutex<Arc<RunState>>>) -> MutexGuard<'_, Arc<RunState>> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
