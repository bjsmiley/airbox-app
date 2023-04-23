use crate::conf;
use tokio::sync::mpsc;

pub struct Node {
    conf: conf::NodeConfig,
    store: conf::NodeConfigStore,

    // a channel for the ui to send queries w/ returnable values
    query: (
        mpsc::UnboundedSender<ReturnableMessage<AppCmd>>,
        mpsc::UnboundedReceiver<ReturnableMessage<AppCmd>>,
    ),

    // a channel for the ui to send commands w/ returnable values
    cmd: (
        mpsc::UnboundedSender<ReturnableMessage<AppQuery>>,
        mpsc::UnboundedReceiver<ReturnableMessage<AppQuery>>,
    ),

    // a channel sender for core to send events to the ui
    events: mpsc::Sender<CoreEvent>,
}

impl Node {
    // pub fn init(dir: String) -> Result<(Self, mpsc::Receiver<CoreEvent>), CoreError> {

    //     // build node config from disk or create
    //     let store = conf::NodeConfigStore
    //     // attach secrets to peers
    //     // build p2p
    //     // start event loop (no dont start event loop, let the app start event loop)
    // }

    // called by
    // pub fn start(&mut self) {
    //     loop {
    //         tokio::select! {}
    //     }
    // }
}

// pub enum NodeError {}

// events to be subscribed to by the application ui
pub enum CoreEvent {}

// commands and queries sent from the application layer to core
pub enum AppCmd {
    SetName(String),
}

pub enum AppQuery {}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "key", content = "data")]
// #[ts(export)]
pub enum CoreResponse {
    Ok,
    // ClientGetState(ClientState),
    // Sum(i32),
}

// a wrapper around external input with a returning sender channel for core to respond
#[derive(Debug)]
pub struct ReturnableMessage<D, R = Result<CoreResponse, CoreError>> {
    data: D,
    tx_return: tokio::sync::oneshot::Sender<R>,
}

pub enum CoreError {}

// core controller is passed to the client to communicate with the core which runs in a dedicated thread
pub struct CoreController {
    query_tx: mpsc::UnboundedSender<ReturnableMessage<AppQuery>>,
    command_tx: mpsc::UnboundedSender<ReturnableMessage<AppCmd>>,
}

impl CoreController {
    pub async fn query(&self, query: AppQuery) -> Result<CoreResponse, CoreError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let payload = ReturnableMessage {
            data: query,
            tx_return: tx,
        };

        self.query_tx.send(payload).unwrap_or(());
        rx.await.unwrap()
    }

    pub async fn command(&self, cmd: AppCmd) -> Result<CoreResponse, CoreError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let payload = ReturnableMessage {
            data: cmd,
            tx_return: tx,
        };

        self.command_tx.send(payload).unwrap_or(());
        rx.await.unwrap()
    }
}
