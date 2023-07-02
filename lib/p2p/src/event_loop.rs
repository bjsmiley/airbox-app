use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::mpsc::{Receiver, UnboundedReceiver},
};
use tracing::debug;

use crate::{
    event::{DiscoveryEvent, InternalEvent},
    manager::P2pManager,
};

pub(crate) async fn p2p_event_loop(
    manager: Arc<P2pManager>,
    mut discovery: Receiver<(DiscoveryEvent, SocketAddr)>,
    mut internal_channel: UnboundedReceiver<InternalEvent>,
    listener: TcpListener,
) {
    loop {
        tracing::debug!(
            "is {} discovery closed? {}",
            manager.id,
            manager.is_discovery_channel_closed()
        );

        tokio::select! {
            internal_event = internal_channel.recv() => {
                let Some(_) = internal_event else {
                    debug!("App stopped sending main event loop messages");
                    break;
                };
            },

            stream_event = listener.accept() => {
                let Ok((stream, addr)) = stream_event else {
                   continue;
                };
                debug!("Remote peer attempting to connect at {:?}", &addr);
                let manager = manager.clone();
                tokio::spawn(async move {
                    if let Ok(peer) = crate::net::accept(&manager, stream).await {
                        manager.handle_new_connection(peer);
                    }
                });
            },
            discovery_event = discovery.recv() => {
                tracing::debug!(
                    "is {} discovery closed? {}",
                    manager.id,
                    manager.is_discovery_channel_closed()
                );
                let Some(event) = discovery_event else {
                    tracing::warn!("Discovery Reciever closed for peer {}", manager.id);
                    break
                    // continue;
                };
                match event {
                    (DiscoveryEvent::PresenceResponse(peer), _) => {
                        if manager.id == peer.id {
                            // the node received its own presence response
                            continue;
                        }
                        debug!("Remote peer discovered at {:?}", peer.addr);
                        manager.handle_peer_discovered(peer);
                        // if let Ok(id) = crate::PeerId::from_string(peer.id.clone()) {
                        //     manager.handle_peer_discovered(id, peer, addr);
                        // }
                    },
                    (DiscoveryEvent::PresenceRequest, addr) => {
                        debug!("Remote peer requested presence at {:?}", addr);
                        manager.handle_presence_request().await;
                    }
                }
            },
        }
    }
    debug!("Shutting down p2p event loop");
}
