use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};

use ab_p2p::peer::PeerId;



// pub fn create_discovery_addr() -> (SocketAddr, Ipv4Addr) {

//     let multicast = Ipv4Addr::new(239,255,42,99);
//     assert_eq!(true, multicast.is_multicast());
//     let addr = SocketAddr::V4(SocketAddrV4::new("192.168.88.231".parse().unwrap(), 0));
//     (addr, multicast)
// }

pub fn create_p2p_addr() -> SocketAddr {
    let addr = SocketAddr::V4(SocketAddrV4::new("127.0.0.1".parse().unwrap(), 0));
    addr
}

pub fn create_peer_id_one() -> PeerId {
    PeerId::from_string("0123456789012345678901234567890123456789".to_string()).unwrap()
}

pub fn create_peer_id_two() -> PeerId {
    PeerId::from_string("QWERTYUIOPQWERTYUIOPQWERTYUIOPQWERTYUIOP".to_string()).unwrap()
}