use std::{net::{SocketAddr}, collections::HashSet, sync::Arc};
use serde::{Serialize, Deserialize};
use tokio::{net::TcpStream, io::DuplexStream};

use crate::manager::P2pManager;

use super::{PeerMetadata, PeerId};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KnownPeer {
// 	pub id: PeerId,
// 	// pub metadata: PeerMetadata,
// 	pub auth_secret: String
// }

/// Represents a peer that has been discovered but not paired with.
/// It is called a candidate as it contains all of the information required to connection and pair with the peer.
/// A peer candidate discovered through mDNS may have been modified by an attacker on your local network but this is deemed acceptable as the attacker can only modify primitive metadata such a name or Spacedrive version which is used for pairing.
/// When we initiated communication with the device we will ensure we are talking to the correct device using PAKE (specially SPAKE2) for pairing and verifying the TLS certificate for general communication.
#[derive(Debug, Clone, Serialize, Deserialize)] // TODO: Type
pub struct PeerCandidate {
	pub id: PeerId,
	pub metadata: PeerMetadata,
	pub addresses: HashSet<SocketAddr>,
	pub auth_secret: String
	// pub addresses: Vec<Ipv4Addr>,
	// pub port: u16,
}

impl PeerCandidate {
	pub fn from_metadata(metadata: &PeerMetadata, auth: String) -> Self {
		Self {
			id: metadata.id.clone(),
			addresses: HashSet::new(),
			auth_secret: auth,
			metadata: metadata.clone()
		}
	}
}



/// This emum represents the type of the connection to the current peer.
/// QUIC is a client/server protocol so when doing P2P communication one client will be the server and one will be the client from a QUIC perspective.
/// The protocol is bi-directional so this doesn't matter a huge amount and the P2P library does it's best to hide this detail from the embedding application as thinking about this can be very confusing.
/// The decision for who is the client and server should be treated as arbitrary and shouldn't affect how the protocol operates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionType {
	/// I am the QUIC server.
	Server,
	/// I am the QUIC client.
	Client,
}


/// Represents a currently connected peer. This struct holds the connection as well as any information the network manager may required about the remote peer.
/// It also stores a reference to the network manager for communication back to the [P2PManager].
/// The [Peer] acts as an abstraction above the QUIC connection which could be a client or server so that when building code we don't have to think about the technicalities of the connection.
#[derive(Debug)]
pub struct Peer {
	/// peer_id holds the id of the remote peer. This is their unique identifier.
	pub id: PeerId,
	/// conn_type holds the type of connection that is being established.
	pub conn_type: ConnectionType,
	/// metadata holds the metadata of the remote peer. This includes information such as their display name and version.
	pub metadata: PeerMetadata,
	/// conn holds the quinn::Connection that is being used to communicate with the remote peer. This allows creating new streams.
	pub conn: DuplexStream,
	// manager is a reference to the p2p manager. This is used to ensure the state of managed connections is updated when Peer is dropped
	// manager: Arc<P2pManager>,
}


// impl Drop for Peer {
//     fn drop(&mut self) {
//         self.manager.
//     }

// }

// impl Debug for Peer {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
// 		f.debug_struct("Peer")
// 			.field("id", &self.id)
// 			.field("conn_type", &self.conn_type)
// 			.field("metadata", &self.metadata)
// 			.finish()
// 	}
// }

impl Peer {
	/// create a new peer from a [quinn::Connection].
	/// Peers can only be created after mutual validation of pair codes
	pub(crate) fn new(
		manager: &Arc<P2pManager>,
		conn_type: ConnectionType,
		conn: TcpStream,
		metadata: PeerMetadata,
	) -> Result<Self, ()> {

		let (transport, application) = tokio::io::duplex(64);

		let id = metadata.id.clone();
		let m = manager.clone();
		tokio::spawn(handler(conn, application, m, id.clone()));

		Ok(Self {
			id,
			conn_type,
			metadata,
			conn: transport,
		})
	}
}

/// continuously running handler for transporting data between local host & connected client
async fn handler(conn: TcpStream, app: DuplexStream, manager: Arc<P2pManager>, id: PeerId) {
	
	let (mut transport_reader, mut transport_writer) = tokio::io::split(conn);
	let (mut app_reader, mut app_writer) = tokio::io::split(app);

	loop{
		tokio::select! {
			result = tokio::io::copy(&mut transport_reader, &mut app_writer) => {
				match result {
					Ok(0) => {
						tracing::debug!("transport buffer drained");
						break;
					}
					Err(e) => {
						tracing::error!("error occured writing data to application {:?}", e);
						break;
					}
					_ => {}
				}
			},
			result = tokio::io::copy(&mut app_reader, &mut transport_writer) => {
				match result {
					Ok(0) => {
						tracing::debug!("application buffer drained");
						break;
					}
					Err(e) => {
						tracing::error!("error occured writing data to transport {:?}", e);
						break;
					}
					_ => {}
				}
			}
		}
	}
	manager.peer_disconnected(&id);
}


// #[cfg(test)]
// mod tests {
//     use std::net::{SocketAddrV4, Ipv4Addr};

//     use dashmap::{DashMap, DashSet};
//     use tokio::sync::mpsc;

//     use crate::{manager::P2pManager, peer::{PeerId, PeerMetadata}};


// 	#[test]
// 	fn peer_data_is_transfered() {
// 		let remote_id = PeerId::from_string("9876543210987654321098765432109876543210".to_string()).unwrap();
// 		let (tx_dsicovery, _) = mpsc::channel(0);
// 		let (tx_internal, _) = mpsc::unbounded_channel();
// 		let (tx_app, _) = mpsc::unbounded_channel();

// 		let manager = P2pManager {
// 			id: PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap(),
// 			metadata: PeerMetadata {
// 				name: "test phone".to_string(),
// 				typ: crate::peer::DeviceType::AppleiPhone,
// 				id: PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap(),
// 				addr: std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST,0))
// 			},
// 			known_peers: DashMap::new(),
// 			discovered_peers: DashMap::new(),
// 			connected_peers: DashSet::new(),
// 			discovery_channel: tx_dsicovery,
// 			internal_channel: tx_internal,
// 			app_channel: tx_app
// 		};
		
// 	}

// }

