use std::fmt::Display;

use crate::peer;

/// P2p Events that get sent to the application
#[derive(Debug)]
pub enum P2pEvent {
    /// A peer was discovered
    PeerDiscovered(peer::PeerMetadata),

    /// A peer connected
    PeerConnected(peer::Peer),

    /// A peer disconnected
    PeerDisconnected(peer::PeerId),
}

#[derive(Debug)]
/// Events being sent and recieved to the discovery mechanism
pub enum DiscoveryEvent {
    /// Request for any presence information
    PresenceRequest,

    /// Response to any presence request
    PresenceResponse(peer::PeerMetadata),
}

impl Display for DiscoveryEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DiscoveryEvent::PresenceRequest => write!(f, "PresenceRequest"),
            DiscoveryEvent::PresenceResponse(_) => write!(f, "PresenceResponse"),
        }
    }
}

impl crate::proto::Frame for DiscoveryEvent {
    fn len(&self) -> u16 {
        match self {
            DiscoveryEvent::PresenceRequest => 1,
            DiscoveryEvent::PresenceResponse(meta) => {
                1 + 2
                    + 2
                    + u16::try_from(meta.name.len()).unwrap()
                    + 40
                    + 2
                    + u16::try_from(meta.addr.to_string().len()).unwrap()
            }
        }
    }
}

pub enum InternalEvent {}
