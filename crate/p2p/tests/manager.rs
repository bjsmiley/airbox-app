use std::{error::Error, net::SocketAddrV4, time::Duration};

use p2p::{
    event::P2pEvent,
    manager::{P2pConfig, P2pManager},
    pairing::PairingAuthenticator,
    peer::{ConnectionType, PeerCandidate},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, timeout};
use tracing::Level;

use crate::common::*;

mod common;

#[tokio::test]
async fn peers_discover_connect_send_data() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_thread_ids(true)
        .init();

    let shared_secret = b"123ABCThisIsSuperSecretShhhh!";
    let auth_a = PairingAuthenticator::new(shared_secret.to_vec())?;
    let auth_b = PairingAuthenticator::new(shared_secret.to_vec())?;

    // node A setup
    let config = P2pConfig {
        id: create_peer_id_one(),
        device: p2p::peer::DeviceType::Windows10Desktop,
        name: String::from("Tester's laptop"),
        multicast: create_multicast_addr(),
        p2p_addr: create_p2p_addr(),
    };
    let (manager_a, mut rx_a) = P2pManager::new(config).await?;

    // node B setup
    let config = P2pConfig {
        id: create_peer_id_two(),
        device: p2p::peer::DeviceType::AppleiPhone,
        name: String::from("Tester's phone"),
        multicast: create_multicast_addr(),
        p2p_addr: create_p2p_addr(),
    };
    let (manager_b, mut rx_b) = P2pManager::new(config).await?;

    // subscribe to node B
    let a = manager_a.get_metadata();
    let b = manager_b.get_metadata();
    manager_a.add_known_peer(PeerCandidate::new(b, auth_b));
    manager_b.add_known_peer(PeerCandidate::new(a, auth_a));

    // node A sends presence request
    sleep(Duration::from_millis(100)).await;
    manager_a.request_presence().await;
    sleep(Duration::from_millis(100)).await;

    // assert node a discovered node b
    let Ok(Some(P2pEvent::PeerDiscovered(metadata))) = timeout(Duration::from_millis(100), rx_a.recv()).await else {

        assert!(false, "node a did not discover node b");
        return Ok(());
    };
    assert!(manager_a.is_discovered(&metadata.id));
    let metadata_b = manager_b.get_metadata();
    assert_eq!(metadata_b.clone(), metadata);

    // assert node a can connect to node b
    let Ok(connected) = timeout(Duration::from_millis(10000),manager_a.connect_to_peer(&metadata.id)).await else {
        assert!(false, "node a did not connect to node b");
        return Ok(());
    };
    let mut proxy_to_b = connected?;
    assert!(manager_a.is_connected(&metadata_b.id));

    let Ok(Some(P2pEvent::PeerConnected(mut proxy_to_a))) = timeout(Duration::from_millis(1000), rx_b.recv()).await else {
        assert!(false, "node b did not connect to node a");
        return Ok(());
    };
    let metadata_a = manager_a.get_metadata();
    assert!(manager_b.is_connected(&metadata_a.id));

    // assert connection types
    assert_eq!(ConnectionType::Client, proxy_to_b.conn_type);
    assert_eq!(ConnectionType::Server, proxy_to_a.conn_type);

    // assert node A can send to node B
    let mut buffer: [u8; 10] = [0; 10];

    proxy_to_b.conn.write_all(b"PING").await?;
    let len = proxy_to_a.conn.read(&mut buffer[..]).await?;
    assert_eq!(b"PING"[..], buffer[..len]);

    // assert node B can send to node A
    proxy_to_a.conn.write_all(b"PONG").await?;
    let len = proxy_to_b.conn.read(&mut buffer[..]).await?;
    assert_eq!(b"PONG"[..], buffer[..len]);

    // assert node A informs when node B disconnects
    drop(proxy_to_a);
    let Ok(Some(P2pEvent::PeerDisconnected(disconnect_id))) = timeout(Duration::from_millis(100), rx_a.recv()).await else {
        assert!(false, "node a did not recieve disconnect event");
        return Ok(());
    };
    assert_eq!(metadata_b.id, disconnect_id);
    assert!(!manager_b.is_connected(&metadata_a.id));
    assert!(!manager_a.is_connected(&metadata_b.id));

    // TODO: assert error or 0 bytes read and written
    // let Err(_) = proxy_to_b.conn.write(b"PONG").await else {
    //     assert!(false, "the tcp connection should fail writes");
    //     return Ok(());
    // };
    // let Err(_) = proxy_to_b.conn.read(&mut buffer).await else {
    //     assert!(false, "the tcp connection should fail writes");
    //     return Ok(());
    // };

    Ok(())
}
