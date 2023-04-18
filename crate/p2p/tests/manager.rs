use std::{time::Duration, error::Error};

use ab_p2p::{manager::{P2pConfig, P2pManager, AppEvent}, discovery::DISCOVERY_MULTICAST, peer::{PeerCandidate, ConnectionType}};
use tokio::time::{sleep, timeout};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{Level, info};

use crate::common::*;

mod common;





#[tokio::test]
async fn peers_discover_connect_send_data() -> Result<(), Box<dyn Error>> {

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_thread_ids(true)
        //.with_file(true)
        //.with_line_number(true)
        .init();

    let shared_secret = "123ABCThisIsSuperSecretShhhh!";

    // node A setup
    let discovery_addr = DISCOVERY_MULTICAST;
    let config = P2pConfig {
        id: create_peer_id_one(),
        device: ab_p2p::peer::DeviceType::Windows10Desktop,
        name: String::from("Tester's laptop"),
        discovery_addr,
        discovery_port: 50692,
        addr_port: 50692,
        p2p_addr: create_p2p_addr()
    };
    let (manager_a, mut rx_a) = P2pManager::new(config).await?;

    // node B setup
    let discovery_addr = DISCOVERY_MULTICAST;
    let config = P2pConfig {
        id: create_peer_id_two(),
        device: ab_p2p::peer::DeviceType::AppleiPhone,
        name: String::from("Tester's phone"),
        discovery_addr,
        discovery_port: 50692,
        addr_port: 50692,
        p2p_addr: create_p2p_addr()
    };
    let (manager_b, mut rx_b) = P2pManager::new(config).await?;

    // subscribe to node B
    let a = manager_a.get_metadata();
    let b = manager_b.get_metadata();
    manager_a.add_known_peer(PeerCandidate::from_metadata(b, shared_secret.clone().to_string()));
    manager_b.add_known_peer(PeerCandidate::from_metadata(a, shared_secret.clone().to_string()));


    // node A sends presence request
    sleep(Duration::from_millis(100)).await;
    manager_a.request_presence().await;
    sleep(Duration::from_millis(100)).await;
    // manager_a.request_presence().await;
    // sleep(Duration::from_millis(100)).await;
    // manager_a.request_presence().await;
    // sleep(Duration::from_millis(100)).await;

    // assert node a discovered node b
    let Ok(Some(AppEvent::PeerDiscovered(metadata))) = timeout(Duration::from_millis(100), rx_a.recv()).await else {

        assert!(false, "node a did not discover node b");
        return Ok(());
    };
    assert!(manager_a.is_discovered(&metadata.id));

    // if let Ok(event) = rx_b.try_recv() {
    //     assert!(false, "node b should not receive an app event: {:?}", event);
    //     return Ok(());
    // };
    let metadata_b = manager_b.get_metadata();
    assert_eq!(metadata_b.clone(), metadata);

    // assert node a can connect to node b
    let Ok(connected) = timeout(Duration::from_millis(10000),manager_a.connect_to_peer(&metadata.id)).await else {
        assert!(false, "node a did not connect to node b");
        return Ok(());
    };
    let mut proxy_to_b = connected?;
    assert!(manager_a.is_connected(&metadata_b.id));

    let Ok(Some(AppEvent::PeerConnected(mut proxy_to_a))) = timeout(Duration::from_millis(1000), rx_b.recv()).await else {
        assert!(false, "node b did not connect to node a");
        return Ok(());
    };
    let metadata_a = manager_a.get_metadata();
    assert!(manager_b.is_connected(&metadata_a.id));

    assert_eq!(ConnectionType::Client, proxy_to_b.conn_type);
    assert_eq!(ConnectionType::Server, proxy_to_a.conn_type);
    let mut buffer: [u8; 10] = [0; 10];

    proxy_to_b.conn.write_all(b"PING").await?;
    let len = proxy_to_a.conn.read(&mut buffer[..]).await?;
    assert_eq!(b"PING"[..], buffer[..len]);

    proxy_to_a.conn.write_all(b"PONG").await?;
    let len = proxy_to_b.conn.read(&mut buffer[..]).await?;
    assert_eq!(b"PONG"[..], buffer[..len]);

    
    drop(proxy_to_a);

    let Ok(Some(AppEvent::PeerDisconnected(disconnect_id))) = timeout(Duration::from_millis(100), rx_a.recv()).await else {
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