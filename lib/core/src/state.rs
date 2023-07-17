use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::proto::Session;

#[derive(Debug, Default)]
pub(crate) struct State {
    /// Cancellation token for discovery background task
    pub discovery_ct: Option<CancellationToken>,
    /// Map of session senders
    pub sessions: HashMap<u64, Sender<Session>>,
    /// An incrementing id for each unique session started with a remote node
    pub session_id: u64,
}
