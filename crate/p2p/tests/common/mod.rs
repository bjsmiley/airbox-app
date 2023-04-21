use std::net::{SocketAddr, SocketAddrV4};

use ab_p2p::peer::PeerId;

pub fn create_p2p_addr() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new("127.0.0.1".parse().unwrap(), 0))
}

pub fn create_peer_id_one() -> PeerId {
    PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap()
}

pub fn create_peer_id_two() -> PeerId {
    PeerId::from_string("QWERTYUIOPQWERTYUIOPQWERTYUIOPQWERTYUIOP".to_string()).unwrap()
}
