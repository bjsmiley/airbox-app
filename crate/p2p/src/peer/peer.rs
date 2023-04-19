use std::{net::{SocketAddr}, collections::HashSet, sync::Arc};
use serde::{Serialize, Deserialize};
use tokio::{net::TcpStream, io::DuplexStream};

use crate::manager::P2pManager;

use super::PeerId;


/// Represents public metadata about a peer. This is designed to hold information which is required among all applications using the P2P library.
/// This metadata is discovered through the discovery process or sent by the connecting device when establishing a new P2P connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerMetadata {
	// pub name: String,
	// pub operating_system: Option<OperationSystem>,
	// pub version: Option<String>,
    pub name: String,
    pub typ: DeviceType,
    pub id: PeerId,
    pub addr: std::net::SocketAddr
    //pub ip: String,
    //pub port: u16
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u16)]
pub enum DeviceType {
    // XboxOne = 1,
    AppleiPhone = 6,
    AppleiPad = 7,
    AndroidDevice = 8,
    Windows10Desktop = 9,
    // Windows10Phone = 11,
    LinuxDevice = 12,
    // WindowsIoT = 13,
    // SurfaceHub = 14,
    WindowsLaptop = 15,
    // WindowsTablet = 16
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KnownPeer {
// 	pub id: PeerId,
// 	// pub metadata: PeerMetadata,
// 	pub auth_secret: String
// }

/// Represents a peer that has been discovered but not paired with.
/// It is called a candidate as it contains all of the information required to connection and pair with the peer.
/// A peer candidate discovered through multicast may have been modified by an attacker on your local network but this is 
/// deemed acceptable as the attacker can only modify primitive metadata such a name or device type.
/// When we initiated communication with the device we will ensure we are talking to the correct device using
/// TOTP not PAKE (specially SPAKE2) for pairing and verifying the TLS (soon) certificate for general communication.
#[derive(Debug, Clone, Serialize, Deserialize)] // TODO: Type
pub struct PeerCandidate {
	pub id: PeerId,
	pub metadata: PeerMetadata,
	pub addresses: HashSet<SocketAddr>,
	pub auth_secret: String
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
	/// I am the QUIC (soon) server.
	Server,
	/// I am the QUIC (soon) client.
	Client,
}


/// Represents a currently connected peer. This struct holds the connection as well as any information 
/// the network manager may required about the remote peer.
/// It also stores a reference to the network manager for communication back to the [P2PManager].
/// The [Peer] acts as an abstraction above the QUIC (soon) connection which could be a client 
/// or server so that when building code we don't have to think about the technicalities of the connection.
#[derive(Debug)]
pub struct Peer {
	/// peer_id holds the id of the remote peer. This is their unique identifier.
	pub id: PeerId,

	/// conn_type holds the type of connection that is being established.
	pub conn_type: ConnectionType,

	/// metadata holds the metadata of the remote peer. This includes information such as their display name and version.
	pub metadata: PeerMetadata,

	/// conn holds the connection that is being used to communicate with the remote peer. This allows creating new streams.
	pub conn: DuplexStream,

	// manager is a reference to the p2p manager. This is used to ensure the state of managed connections is updated when Peer is dropped
	// manager: Arc<P2pManager>,
}

impl Peer {
	/// create a new peer from a network connection.
	/// Peers can only be created after mutual validation of pairing codes
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

/// continuously running handler for transporting data between local peer & remote peer
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