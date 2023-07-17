use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

pub static DISCOVERY_MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 98);

pub fn multicast(
    addr: &SocketAddr,
    multi_addr: &SocketAddr,
) -> Result<(UdpSocket, SocketAddr), std::io::Error> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    if let (SocketAddr::V4(a), SocketAddr::V4(m)) = (addr, multi_addr) {
        socket.join_multicast_v4(m.ip(), a.ip())?
    }
    socket.set_nonblocking(true)?;
    Ok((UdpSocket::from_std(socket.into())?, *multi_addr))
}
