use futures::{SinkExt, StreamExt};
use p2p::peer::Peer;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, BufReader, BufStream},
    sync::{
        mpsc::{self, UnboundedSender},
        oneshot,
    },
};
use tokio_util::{
    codec::{
        AnyDelimiterCodec, AnyDelimiterCodecError, Decoder, Encoder, Framed, FramedRead,
        FramedWrite,
    },
    io::SyncIoBridge,
};
use tracing::{debug, error};

use crate::{
    api::cmd::PeerRequest,
    node::InternalEvent,
    proto::{Session, SessionCodec},
};

// #[derive(Debug, Serialize, Deserialize)]
// pub enum PeerRequest {
//     OpenUri(String),
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum PeerResponse {
//     Ok,
//     Err,
//     // Waiting,
//     // accept, complete, waiting
// }

pub(crate) async fn client_handler(peer: Peer, req: Session, tx: UnboundedSender<InternalEvent>) {
    let (r, w) = tokio::io::split(peer.conn);
    let mut reader = FramedRead::new(r, SessionCodec::default());
    let mut writer = FramedWrite::new(w, SessionCodec::default());

    debug!("Starting session as client with peer {}", peer.metadata.id);
    if writer.send(req).await.is_err() {
        error!("Failed to send outbound request.");
        return;
    }

    while let Some(Ok(session)) = reader.next().await {
        if tx
            .send(InternalEvent::SessionResult {
                id: peer.id.clone(),
                body: session,
            })
            .is_err()
        {
            debug!("Failed to handle inbound response.");
            break;
        }
    }
    debug!("Ending session as client with peer {}", peer.metadata.id);
}

pub(crate) async fn server_handler(peer: Peer, tx: UnboundedSender<InternalEvent>) {
    let (r, w) = tokio::io::split(peer.conn);
    let mut reader = FramedRead::new(r, SessionCodec::default());
    let mut writer = FramedWrite::new(w, SessionCodec::default());

    while let Some(Ok(session)) = reader.next().await {
        debug!("Accepting session as server with peer {}", peer.metadata.id);
        let mut mpsc = mpsc::channel(64);
        if tx
            .send(InternalEvent::InboundSession {
                meta: peer.metadata.clone(),
                body: session,
                tx: mpsc.0,
            })
            .is_err()
        {
            debug!("Failed to handle inbound request.");
            break;
        };
        while let Some(res) = mpsc.1.recv().await {
            if writer.send(res).await.is_err() {
                error!("Failed to send outbound response.");
                // TODO: send err to client
                break;
            }
        }

        debug!("Ending session as server with peer {}", peer.metadata.id);
    }
}
