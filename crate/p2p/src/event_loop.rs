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
        tokio::select! {
            discovery_event = discovery.recv() => {
                let Some(event) = discovery_event else {
                    debug!("Discovery stopped sending main event loop messages");
                    break
                };
                match event {
                    (DiscoveryEvent::PresenceResponse(peer), _) => {
                        if manager.id == peer.id {
                            // the node received its own presence response
                            continue;
                        }
                        debug!("Peer discovered at {:?}", peer.addr);
                        manager.handle_peer_discovered(peer);
                        // if let Ok(id) = crate::PeerId::from_string(peer.id.clone()) {
                        //     manager.handle_peer_discovered(id, peer, addr);
                        // }
                    },
                    (DiscoveryEvent::PresenceRequest, addr) => {
                        debug!("Peer requested presence at {:?}", addr);
                        manager.handle_presence_request().await;
                    }
                }
            },
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
                debug!("Peer attempting to connect at {:?}", &addr);
                let manager = manager.clone();
                tokio::spawn(async move {
                    if let Ok(peer) = crate::net::accept(&manager, stream).await {
                        manager.handle_new_connection(peer);
                    }
                });
            }
        }
    }
    debug!("Shutting down p2p event loop");
}
