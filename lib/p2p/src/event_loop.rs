use futures_util::{SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::mpsc::UnboundedReceiver,
};
use tokio_util::udp::UdpFramed;
use tracing::{debug, error};

use crate::{
    event::{DiscoveryEvent, InternalEvent},
    manager::P2pManager,
    proto::DiscoveryCodec,
};

pub(crate) async fn p2p_event_loop(
    manager: Arc<P2pManager>,
    mut internal_channel: UnboundedReceiver<InternalEvent>,
    mut discovery_channel: UnboundedReceiver<DiscoveryEvent>,
    listener: TcpListener,
    discovery: (UdpSocket, SocketAddr),
) {
    let (mut udp_tx, mut udp_rx) = UdpFramed::new(discovery.0, DiscoveryCodec).split();

    loop {
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
            outbound_discovery = discovery_channel.recv() => {
                let Some(event) = outbound_discovery else {
                    debug!("App stopped sending main event loop messages");
                    break;
                };
                debug!("Sending {} to addr {}", event, discovery.1);
                if let Err(e) = udp_tx.send((event, discovery.1)).await {
                    error!("Error sending discovery event: {:?}", e);
                };
            },
            inbound_discovery = udp_rx.next() => {
                let Some(frame) = inbound_discovery else {
                    error!("Recieved None from inbound discovery");
                    break;
                };
                match frame {
                    Err(e) => error!("error reading from Discovery: {:?}", e),
                    Ok((DiscoveryEvent::PresenceResponse(peer), addr)) => {
                        if manager.id != peer.id {
                            debug!("Remote peer discovered at {:?}", addr);
                            manager.handle_peer_discovered(peer);
                        }
                    },
                    Ok((DiscoveryEvent::PresenceRequest(dedup), addr)) => {
                        if manager.dedup != dedup {
                            debug!("Remote peer requested presence at {:?}", addr);
                            manager.handle_presence_request();
                        }
                    }
                }
            }
        }
    }
    debug!("Shutting down p2p event loop");
}
