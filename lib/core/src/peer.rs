use futures::{SinkExt, StreamExt};
use p2p::peer::Peer;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error};

use crate::{
    err,
    node::InternalEvent,
    proto::{Session, SessionCodec},
    store,
};

impl store::Persistable for p2p::peer::Identity {
    type Error = err::CoreError;

    fn read<R>(r: R) -> Result<Self, Self::Error>
    where
        R: std::io::Read,
    {
        Ok(serde_json::from_reader(r)?)
    }

    fn write<W>(&self, w: &mut W) -> Result<(), Self::Error>
    where
        W: std::io::Write,
    {
        let json = serde_json::to_string(self)?;
        w.write_all(json.as_bytes())?;
        Ok(())
    }
}

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
