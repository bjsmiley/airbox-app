use std::{sync::Arc, net::{SocketAddr, SocketAddrV4, Ipv4Addr, IpAddr}, collections::HashSet};

use dashmap::{DashMap, DashSet};
use thiserror::Error;
use tokio::{sync::mpsc, net::{UdpSocket, TcpListener, TcpStream}};
use tracing::{debug, error};

use crate::{peer::{PeerId, PeerMetadata, PeerCandidate, Peer, DeviceType, PeerIdError}, discovery::{DiscoveryEvent, discovery, self, create_duplex_multicast_socket}, event_loop::p2p_event_loop};

pub struct P2pManager {

    // store internal state

    /// PeerId is the unique identifier of the current peer.
    pub(crate) id: PeerId,
    
    // /// identity is the TLS identity of the current peer.
	// pub(crate) identity: (Certificate, PrivateKey),
    
    /// The metadata of the current peer
    pub(crate) metadata: PeerMetadata,

    /// known_peers are peers who have been previously paired up with, only from these peers can the 
    /// P2p Manager discover and connect with.
    known_peers: DashMap<PeerId, PeerCandidate>,

    /// discovered_peers contains a list of all peers which have been discovered by any discovery mechanism.
    discovered_peers: DashMap<PeerId, PeerCandidate>,
    
    /// connected_peers
    connected_peers: DashSet<PeerId>,

    /// channel to send Discovery events
    discovery_channel: mpsc::Sender<DiscoveryEvent>,

    /// internal_channel is a channel which is used to communicate with the main internal event loop.
	internal_channel: mpsc::UnboundedSender<InternalEvent>,

    /// app_channel is a channel which is used to communicate with the application
    app_channel: mpsc::UnboundedSender<AppEvent>

}

impl P2pManager {
    pub async fn new(config: P2pConfig) -> Result<(Arc<Self>, mpsc::UnboundedReceiver<AppEvent>), P2pError> {
        
        // let peer_id = PeerId::from_string(config.id.clone())?;
        
        // setup multicast udp
        // let discover = {
        //     let bind_addr = config.discovery_addr.clone();
        //     let multicast = config.multicast_ip.clone();
        //     if !multicast.is_multicast() { return Err(P2pError::InvalidMulticastAddr); }
        //     let sock = UdpSocket::bind(bind_addr).await?;
        //     let IpAddr::V4(interface) = bind_addr.ip() else {
        //         error!("no way");
        //         panic!();
        //     };
        //     sock.
        //     sock.join_multicast_v4(multicast, interface)?;
        //     debug!("Multicast address: {}", multicast);
        //     debug!("Udp socket: {}", sock.local_addr()?);
        //     discovery(sock, bind_addr)
        // };

        // let discover = {
        //     let addr = config.multicast_ip.clone();
        //     if !addr.is_multicast() { return Err(P2pError::InvalidMulticastAddr); }
        //     let sock = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)).await?;
        //     // let IpAddr::V4(interface) = bind_addr.ip() else {
        //     //     error!("no way");
        //     //     panic!();
        //     // };
        //     sock.join_multicast_v4(addr, Ipv4Addr::UNSPECIFIED)?;
        //     let finally = sock.local_addr()?;
        //     debug!("Multicast address: {}", addr);
        //     debug!("Udp socket: {}", finally);
        //     discovery(sock, SocketAddr::V4(SocketAddrV4::new(addr, 5545)))
        // };

        let discover = {
            // let (socket, multi_addr) = create_duplex_multicast_socket(&Ipv4Addr::UNSPECIFIED,
                let (socket, multi_addr) = create_duplex_multicast_socket(
                    // &"192.168.88.231".parse().unwrap(),
                    &"127.0.0.1".parse().unwrap(),

                    config.addr_port,
                 &config.discovery_addr, 
                 config.discovery_port)?;
            discovery(socket, multi_addr)
        };

        // setup tcp listener
        let listener = TcpListener::bind(config.p2p_addr.clone()).await?;
        debug!("Peer {} listening on {}", config.id.clone(), listener.local_addr()?);

        // setup metadata
        let metadata = PeerMetadata {
            id: config.id.clone(),
            typ: config.device,
            name: config.name,
            addr: listener.local_addr()?
            // ip: listener.local_addr()?.ip().to_string(),
            // port: listener.local_addr()?.port()
        };

        let internal_channel = mpsc::unbounded_channel();
        let app_channel = mpsc::unbounded_channel();


        let this = Arc::new(Self {
            id: config.id,
            metadata,
            known_peers: DashMap::new(),
            discovered_peers: DashMap::new(),
            connected_peers: DashSet::new(),
            discovery_channel: discover.0,
            internal_channel: internal_channel.0,
            app_channel: app_channel.0
        });

        tokio::spawn(p2p_event_loop(this.clone(),
         discover.1,
         internal_channel.1,
        listener));

        Ok((this, app_channel.1))
    }

    /// called by the application to populate already known peers
    pub fn add_known_peer(&self, peer: PeerCandidate) {
        self.known_peers.insert(peer.id.clone(), peer);
    }

    // called by the application to send a presenct request
    pub async fn request_presence(&self) {
        if let Err(e) = self.discovery_channel.send(DiscoveryEvent::PresenceRequest).await {
            error!("application is unable to request presence: {}", e);
        }
        // debug!("peer is emitting presence request");

    }

    // application calls this to get local metadata
    pub fn get_metadata(&self) -> &PeerMetadata {
        &self.metadata
    }

    pub fn is_discovered(&self, id: &PeerId) -> bool {
        self.discovered_peers.contains_key(id)
    }

    pub fn is_connected(&self, id: &PeerId) -> bool {
        self.connected_peers.contains(id)
    }

    /// application calls this to connect to a peer
    pub async fn connect_to_peer(self: &Arc<Self>, id: &PeerId) -> Result<Peer,P2pClientConnectError> {
        if self.connected_peers.contains(id) { return Err(P2pClientConnectError::AlreadyConnected); }
        let Some(candidate) = self.discovered_peers.get(id) else {
            return Err(P2pClientConnectError::NotFound)
        };

        // let peer = candidate.clone();

        for addr in &candidate.addresses {
            match TcpStream::connect(addr).await {
                Err(e) => {
                    error!("Attempt to connect to address {:?} failed {:?}", addr, e);
                }
                Ok(conn) => {
                    debug!("Attempting to connect to {:?}", addr);
                    let peer = crate::net::connect(&self, conn, &candidate).await?;
                    self.connected_peers.insert(id.clone());
                    return Ok(peer);
                }
            }
        }
        Err(P2pClientConnectError::Address)


    }

    // [START] Crate methods the event loop can call

    /// called by a connected peer's connection handler when closing
    pub(crate) fn peer_disconnected(self: &Arc<Self>, id: &PeerId) {
        self.connected_peers.remove(id);
        if let Err(_) = self.app_channel.send(AppEvent::PeerDisconnected(id.clone())) {
            error!("failed to send PeerDisconnected event to the application");
        }
    }

    /// called by host handshake to attempt to get the PeerCandidate
    pub(crate) fn get_peer_candidate(&self, id: &PeerId) -> Option<PeerCandidate> {
        self.discovered_peers
            .get(id)
            .map(|p| p.value().clone() )
            .or(self.known_peers.get(id)
            .map(|p| p.value().clone()))
    }

    /// event loop calls this to determine if incoming connection is from a discovered peer
    // pub(crate) fn get_known_or_discovered_peer_by_addr(&self, addr: &SocketAddr) -> Option<PeerCandidate> {
    //     let Some(peer) = self.discovered_peers.iter().find(|p| p.addresses.contains(&addr)) else {
    //         return None;
    //     };
    //     Some(peer.value().clone())
    // }




    /// event loop calls this to inform manager a peer was discovered
    pub(crate) fn handle_peer_discovered(&self, peer: PeerMetadata) {
        let id = peer.id.clone();
        if !self.connected_peers.contains(&id) && 
           !self.discovered_peers.contains_key(&id) {
            if let Some(known) = self.known_peers.remove(&id) {
                let mut candidate = PeerCandidate {
                    id: id.clone(),
                    metadata: peer.clone(),
                    addresses: HashSet::new(),
                    auth_secret: known.1.auth_secret
                };
                candidate.addresses.insert(peer.addr.clone());
                self.discovered_peers.insert(id.clone(), candidate.clone());
                self.known_peers.insert(id, candidate.clone());
                debug!("discovered peer is recorded");
                if let Err(_) = self.app_channel.send(AppEvent::PeerDiscovered(candidate.metadata)) {
                    error!("failed to send PeerDiscovered event to the application");
                };
                return;
            }
        }
    }

    /// event loop calls this to inform manager a peer requested our precesence
    pub(crate) async fn handle_presence_request(&self) {
        if let Err(e) = self.discovery_channel.send(DiscoveryEvent::PresenceResponse(self.metadata.clone())).await {
            error!("event loop is unable to emit presence: {}", e);
        }
        debug!("peer is emitting presence");
    }

    /// event loop calls this to inform manager a peer is now connected
    pub(crate) fn handle_new_connection(&self, peer: Peer) {
        let id = peer.id.clone();
        self.connected_peers.insert(id.clone());
        if let Err(_) = self.app_channel.send(AppEvent::PeerConnected(peer)) {
            error!("failed to send PeerConnected event to the application");
        };
    }
    // [ END ] Crate methods the event loop can call

}


#[derive(Debug)]
pub enum AppEvent {
    PeerDiscovered(PeerMetadata),
    PeerConnected(Peer),
    PeerDisconnected(PeerId)
}

pub enum InternalEvent {

}

pub struct P2pConfig {
    pub id: PeerId,
    pub device: DeviceType,
    pub name: String,
    pub addr_port: u16,
    pub discovery_port: u16,
    pub discovery_addr: Ipv4Addr,
    pub p2p_addr: SocketAddr,
}

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("Invalid Peer Id")]
    InvalidPeerId(#[from] PeerIdError),
    #[error("The address for discovery is not a multicast address")]
    InvalidMulticastAddr,
    #[error("Could not start discovery")]
    Discovery(#[from] discovery::CreateSocketError),
    #[error("Tokio io error")]
    Tokio(#[from] tokio::io::Error)
}

/// P2p errors as the client trying to connect
#[derive(Debug, Error)]
pub enum P2pClientConnectError {
    #[error("Peer already connected")]
    AlreadyConnected,
    #[error("Peer not found")]
    NotFound,
    #[error("Peer has no connectable addresses")]
    Address,
    #[error("A handshake error occured")]
    Handshake(#[from] crate::net::ConnectHandshakeError)

}