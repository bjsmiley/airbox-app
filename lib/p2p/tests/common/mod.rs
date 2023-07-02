use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use p2p::{discovery::DISCOVERY_MULTICAST, peer::PeerId};

pub fn create_p2p_addr() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
}

pub fn create_multicast_addr() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(DISCOVERY_MULTICAST, 50692))
}

pub fn create_peer_id_one() -> PeerId {
    PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap()
}

pub fn create_peer_id_two() -> PeerId {
    PeerId::from_string("QWERTYUIOPQWERTYUIOPQWERTYUIOPQWERTYUIOP".to_string()).unwrap()
}
