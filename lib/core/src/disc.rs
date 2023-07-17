use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::node::InternalEvent;

/// continuously trigger the main event loop to send presense requests at a 2 second interval
pub(crate) async fn start(tx: UnboundedSender<InternalEvent>, ct: CancellationToken) {
    if tx.send(InternalEvent::RequestPresence).is_err() {
        return;
    }
    while !ct.is_cancelled() {
        tokio::time::sleep(Duration::from_secs(2)).await;
        if tx.send(InternalEvent::RequestPresence).is_err() {
            return;
        }
    }
}
