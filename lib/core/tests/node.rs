use std::time::Duration;

use fdcore::{
    api::{cmd::PeerRequest, event::{CoreEvent, ControlStatus, ControlMessage}},
    node::{Node},
};
use tokio::task::Id;

// #[tokio::test]
// pub async fn node_discovery_test() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt()
//         .with_max_level(tracing::Level::DEBUG)
//         .with_thread_ids(true)
//         .init();

//     fdcore::secret::mock_store();
//     let a = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("a");
//     _ = std::fs::remove_dir_all(a.clone());
//     _ = std::fs::create_dir_all(a.clone());

//     let (mut na, mut nae) = Node::init(a).await?;
//     let nacmd = na.get_cmd_api();
//     let naque = na.get_query_api();
//     tokio::spawn(async move {
//         na.start().await;
//         tracing::info!("Node A stopped");
//     });

//     nacmd
//         .pair(fdcore::node::QrPayload {
//             secret: String::from("123abc7890987654321"),
//             peer: PeerMetadata::default(),
//         })
//         .await
//         .unwrap();

//     nacmd.start_discovery().await.unwrap();
//     tokio::time::sleep(Duration::from_secs(500)).await;
//     Ok(())
// }

#[tokio::test]
pub async fn nodes_pair_send_openuri() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    fdcore::secret::mock_store();
    let a = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("a");
    let b = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("b");
    _ = std::fs::remove_dir_all(a.clone());
    _ = std::fs::remove_dir_all(b.clone());
    _ = std::fs::create_dir_all(a.clone());
    _ = std::fs::create_dir_all(b.clone());

    let (mut na, mut nae) = Node::init(a).await?;
    let (mut nb, mut nbe) = Node::init(b).await?;
    let nacmd = na.get_cmd_api();
    let naque = na.get_query_api();
    let nbcmd = nb.get_cmd_api();
    let nbque = nb.get_query_api();

    tokio::spawn(async move {
        na.start().await;
        tracing::info!("Node A stopped");
    });

    tokio::spawn(async move {
        nb.start().await;
        tracing::info!("Node B stopped");
    });

    // Pair the two nodes
    let qr = naque.get_qrcode().await.unwrap();
    nbcmd.pair(qr.clone()).await.unwrap();
    let qr = nbque.get_qrcode2(qr.secret).await.unwrap();
    nacmd.pair(qr).await.unwrap();

    // confirm both nodes now know each other
    let confa = naque.get_config().await.unwrap();
    let confb = nbque.get_config().await.unwrap();
    assert_eq!(1, confa.known_peers.len());
    assert_eq!(1, confb.known_peers.len());

    // start discovery
    nacmd.start_discovery().await.unwrap();
    nbcmd.start_discovery().await.unwrap();
    tokio::time::sleep(Duration::from_secs(5)).await;
    nacmd.stop_discovery().await.unwrap();
    nbcmd.stop_discovery().await.unwrap();

    let disca = naque.get_discovered_peers().await.unwrap();
    let discb = nbque.get_discovered_peers().await.unwrap();
    assert_eq!(1, disca.len());
    assert_eq!(1, discb.len());
    assert!(disca.iter().any(|p| { p.id == confb.id }));
    assert!(discb.iter().any(|p| { p.id == confa.id }));

    // send a openuri request from node A to node B
    nacmd
        .send_peer(
            confb.id.clone(),
            PeerRequest::LaunchUri(
                "https://www.google.com/search?q=what+is+the+meaning+of+life".to_string(),
            ),
        )
        .await
        .unwrap();
    _ = nae.recv().await; // skip discovery message
    _ = nbe.recv().await; // skip discovery message

    let Some(CoreEvent::AppControlUpdate { peer: id, status: ControlStatus::Waiting }) = nae.recv().await else { 
        panic!("The wrong response was received")
    };
    assert_eq!(id, confb.id);
    let Some(CoreEvent::AppControl { peer: id, ctl: ControlMessage::LaunchUri { uri, ask: true }, sid: s }) = nbe.recv().await else {
        panic!("The wrong response was received")
    };
    assert_eq!(id, confa.id);
    assert_eq!(
        "https://www.google.com/search?q=what+is+the+meaning+of+life",
        uri
    );
    nbcmd.ctl_accept(id, s).await.unwrap();
    let Some(CoreEvent::AppControlUpdate { peer: id, status: ControlStatus::Success }) = nae.recv().await else {
        panic!("The wrong response was received")
    };
    assert_eq!(id, confb.id);
    Ok(())
}
