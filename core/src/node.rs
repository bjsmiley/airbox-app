
use std::net::{SocketAddr, SocketAddrV4};
use std::time::Duration;

use crate::{conf, err, lan::LanManager, plat, secret};

use p2p::{
    discovery,
    event::P2pEvent,
    manager::{P2pConfig, P2pManager},
};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::debug;

pub struct Node {
    conf: conf::NodeConfig,
    store: conf::NodeConfigStore,
    p2p: std::sync::Arc<P2pManager>,
    lan: LanManager,

    // a channel for the ui to send queries w/ returnable values
    query: (
        mpsc::UnboundedSender<ReturnableMessage<AppQuery>>,
        mpsc::UnboundedReceiver<ReturnableMessage<AppQuery>>,
    ),

    // a channel for the ui to send commands w/ returnable values
    cmd: (
        mpsc::UnboundedSender<ReturnableMessage<AppCmd>>,
        mpsc::UnboundedReceiver<ReturnableMessage<AppCmd>>,
    ),

    // a channel for child threads to send events back to the core
    internal: (
        mpsc::UnboundedSender<InternalEvent>,
        mpsc::UnboundedReceiver<InternalEvent>,
    ),

    // a channel sender for core to send events to the ui
    events: mpsc::Sender<CoreEvent>,

    // a channel receiver for core to receive p2p events
    p2p_events: mpsc::UnboundedReceiver<P2pEvent>,
}

impl Node {
    pub async fn init(dir: String) -> Result<(Self, mpsc::Receiver<CoreEvent>), err::CoreError> {
        // build node config from disk or create
        let store: conf::NodeConfigStore = dir.into();
        let conf = store.get()?;

        // build lan
        let lan = LanManager::new()?;

        // build p2p
        let p2p_conf = P2pConfig {
            id: conf.id.clone(),
            device: plat::device_type(),
            name: conf.name.clone(),
            multicast: SocketAddr::V4(SocketAddrV4::new(discovery::DISCOVERY_MULTICAST, 50692)), // TODO 0 port??
            p2p_addr: SocketAddr::V4(SocketAddrV4::new(
                *lan.lan
                    .iter()
                    .next()
                    .ok_or(err::CoreError::NoNetworkAccess)?,
                0,
            )),
        };
        let (p2p, p2p_events) = P2pManager::new(p2p_conf).await?;

        // append known peers
        for p in secret::to_known(&conf.known_peers) {
            p2p.add_known_peer(p);
        }

        let (events, events_rx) = mpsc::channel(64);

        let node = Self {
            conf,
            store,
            p2p,
            lan,
            query: mpsc::unbounded_channel(),
            cmd: mpsc::unbounded_channel(),
            internal: mpsc::unbounded_channel(),
            events,
            p2p_events,
        };

        Ok((node, events_rx))
    }

    // called by
    pub async fn start(&mut self) {
        // TODO: start p2p event loop here?
        loop {
            tokio::select! {
                Some(q) = self.query.1.recv() => {
                    let res = self.handle_query(q.data).await;
                    q.tx_return.send(res).unwrap_or(());
                }
                Some(c) = self.cmd.1.recv() => {
                    let res = self.handle_command(c.data).await;
                    c.tx_return.send(res).unwrap_or(());
                }
                Some(e) = self.internal.1.recv() => self.handle_event(e).await,
                Ok(n) = self.lan.next() => {
                    debug!("LAN event: {:?}", n);
                }
                // Ok(p2p) = self.p2p_events.recv() => {
                //     match p2p {
                //         P2pEvent::PeerDiscovered(metadata)
                //     }
                // }
            }
        }

        // get state from p2p and persist
    }

    // handle queries
    async fn handle_query(&self, _query: AppQuery) -> Result<CoreResponse, err::CoreError> {
        todo!()
    }

    // handle commands
    async fn handle_command(&mut self, cmd: AppCmd) -> Result<CoreResponse, err::CoreError> {
        match cmd {
            AppCmd::Discover(span) => {
                let p2p = self.p2p.clone();
                tokio::spawn(async move {
                    for _ in 0..span {
                        sleep(Duration::from_secs(1)).await;
                        p2p.request_presence().await;
                    }
                });
            }
            AppCmd::SetName(_new) => {
                todo!()
            }
        }
        Ok(CoreResponse::Ok)
    }

    // handle events
    async fn handle_event(&mut self, _event: InternalEvent) {
        todo!()
    }
}

// pub enum NodeError {}

// events to be subscribed to by the application ui
pub enum CoreEvent {
    Discovered(),
}

// commands and queries sent from the application layer to core
pub enum AppCmd {
    SetName(String),
    Discover(u8),
}

pub enum AppQuery {
    GetConf,
}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "key", content = "data")]
// #[ts(export)]
pub enum CoreResponse {
    Ok,
    Conf(conf::NodeConfig), // ClientGetState(ClientState),
                            // Sum(i32),
}

pub(crate) enum InternalEvent {}

// a wrapper around external input with a returning sender channel for core to respond
#[derive(Debug)]
pub struct ReturnableMessage<D, R = Result<CoreResponse, err::CoreError>> {
    data: D,
    tx_return: tokio::sync::oneshot::Sender<R>,
}

// core controller is passed to the client to communicate with the core which runs in a dedicated thread
pub struct CoreController {
    query_tx: mpsc::UnboundedSender<ReturnableMessage<AppQuery>>,
    command_tx: mpsc::UnboundedSender<ReturnableMessage<AppCmd>>,
}

impl CoreController {
    pub async fn query(&self, query: AppQuery) -> Result<CoreResponse, err::CoreError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let payload = ReturnableMessage {
            data: query,
            tx_return: tx,
        };

        self.query_tx.send(payload).unwrap_or(());
        rx.await.unwrap()
    }

    pub async fn command(&self, cmd: AppCmd) -> Result<CoreResponse, err::CoreError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let payload = ReturnableMessage {
            data: cmd,
            tx_return: tx,
        };

        self.command_tx.send(payload).unwrap_or(());
        rx.await.unwrap()
    }
}
