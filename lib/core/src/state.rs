use std::collections::{HashMap, HashSet};

use futures::channel::mpsc::UnboundedSender;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::proto::Session;

// use crate::api::cmd::PeerRequest;

#[derive(Debug, Default)]
pub(crate) struct State {
    /// Cancellation token for discovery background task
    pub discovery_ct: Option<CancellationToken>,
    // /// Map of channel senders
    pub sessions: HashMap<u64, Sender<Session>>,
    pub session_id: u64,
    // Set of peerIds known to this local device but are unaware of this local device
    // pub half_known: HashSet<p2p::peer::PeerId>,
}
