use p2p::peer::PeerId;
use tokio::sync::{mpsc, oneshot};

use crate::err;

use self::cmd::PeerRequest;

pub type QueryMsg = Msg<query::Request, query::Response>;
pub type QueryApi = Api<query::Request, query::Response>;
pub type CmdMsg = Msg<cmd::Request, cmd::Response>;
pub type CmdApi = Api<cmd::Request, cmd::Response>;
pub type ApiResult<R> = Result<R, ()>;
pub type EmptyApiResult = ApiResult<()>;

impl From<cmd::Response> for EmptyApiResult {
    fn from(error: cmd::Response) -> Self {
        match error {
            cmd::Response::Ok => Ok(()),
            // cmd::Response::Err => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Msg<D, R> {
    pub(crate) req: D,
    pub(crate) res: oneshot::Sender<Result<R, err::CoreError>>,
}

#[derive(Clone)]
pub struct Api<D, R> {
    pub(crate) tx: mpsc::UnboundedSender<Msg<D, R>>,
}

impl<D, R> Api<D, R> {
    pub async fn send(&self, req: D) -> Result<R, err::CoreError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let payload = Msg { req, res: tx };

        self.tx.send(payload).unwrap_or(());
        rx.await.unwrap()
    }

    async fn send2(&self, req: D) -> ApiResult<R> {
        self.send(req).await.map_err(|_| ())
    }
}

impl QueryApi {
    pub async fn get_config(&self) -> ApiResult<crate::conf::NodeConfig> {
        match self.send2(query::Request::GetConf).await? {
            query::Response::Conf(conf) => Ok(conf),
            _ => Err(()),
        }
    }

    pub async fn get_discovered_peers(&self) -> ApiResult<Vec<p2p::peer::PeerMetadata>> {
        match self.send2(query::Request::GetDiscoveredPeers).await? {
            query::Response::DiscoveredPeers(peers) => Ok(peers),
            _ => Err(()),
        }
    }

    pub async fn get_qrcode(&self) -> ApiResult<crate::node::QrPayload> {
        match self.send2(query::Request::GetSharableQrCode(None)).await? {
            query::Response::SharableQrCode(code) => Ok(code),
            _ => Err(()),
        }
    }

    pub async fn get_qrcode2(&self, secret: String) -> ApiResult<crate::node::QrPayload> {
        match self
            .send2(query::Request::GetSharableQrCode(Some(secret)))
            .await?
        {
            query::Response::SharableQrCode(code) => Ok(code),
            _ => Err(()),
        }
    }
}

impl CmdApi {
    pub async fn start_discovery(&self) -> EmptyApiResult {
        self.send2(cmd::Request::StartDiscovery).await?.into()
    }

    pub async fn stop_discovery(&self) -> EmptyApiResult {
        self.send2(cmd::Request::StopDiscovery).await?.into()
    }

    pub async fn set_config(&self, config: crate::conf::NodeConfig) -> EmptyApiResult {
        self.send2(cmd::Request::SetConf(config)).await?.into()
    }

    pub async fn pair(&self, payload: crate::node::QrPayload) -> EmptyApiResult {
        self.send2(cmd::Request::Pair(payload)).await?.into()
    }

    pub async fn send_peer(&self, id: PeerId, req: PeerRequest) -> EmptyApiResult {
        self.send2(cmd::Request::SendPeer { peer: id, req })
            .await?
            .into()
    }

    pub async fn ctl_cancel(&self, id: PeerId, session: u64) -> EmptyApiResult {
        self.send2(cmd::Request::Ack {
            peer: id,
            sid: session,
            ack: cmd::Ack::Cancelled,
        })
        .await?
        .into()
    }

    pub async fn ctl_accept(&self, id: PeerId, session: u64) -> EmptyApiResult {
        self.send2(cmd::Request::Ack {
            peer: id,
            sid: session,
            ack: cmd::Ack::Accepted,
        })
        .await?
        .into()
    }
}

pub mod cmd {
    use p2p::peer;
    use serde::{Deserialize, Serialize};

    use crate::proto::{self};

    // commands and queries sent from the application layer to core
    #[derive(Debug, Serialize, Deserialize)]
    pub enum Request {
        SetConf(crate::conf::NodeConfig),
        StartDiscovery,
        StopDiscovery,
        SendPeer {
            peer: peer::PeerId,
            req: PeerRequest,
        },
        // qr code json payload
        Pair(crate::node::QrPayload),
        Ack {
            peer: peer::PeerId,
            sid: u64,
            ack: Ack,
        },
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Ack {
        Accepted,
        // AwaitingUser,
        Cancelled,
        // TimedOut
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Response {
        Ok,
        // Err,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum PeerRequest {
        LaunchUri(String),
    }

    impl Into<proto::CtlRequest> for PeerRequest {
        fn into(self) -> proto::CtlRequest {
            match self {
                PeerRequest::LaunchUri(uri) => proto::CtlRequest::LaunchUri(uri),
            }
        }
    }

    impl Into<proto::CtlResponse> for Ack {
        fn into(self) -> proto::CtlResponse {
            match self {
                Ack::Accepted => proto::CtlResponse::Success,
                // Ack::AwaitingUser => proto::CtlResponse::Waiting,
                Ack::Cancelled => proto::CtlResponse::Cancel,
            }
        }
    }
}

pub mod query {
    use serde::{Deserialize, Serialize};

    use crate::conf;

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Request {
        GetConf,
        GetDiscoveredPeers,
        GetSharableQrCode(Option<String>),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Response {
        Conf(conf::NodeConfig),
        DiscoveredPeers(Vec<p2p::peer::PeerMetadata>),
        SharableQrCode(crate::node::QrPayload),
        // Err,
    }
}

pub mod event {
    use p2p::peer::{PeerId, PeerMetadata};
    use serde::{Deserialize, Serialize};

    // events to be subscribed to by the application ui
    #[derive(Debug, Serialize, Deserialize)]
    pub enum CoreEvent {
        Discovered(PeerMetadata),
        // AskLaunchUri(PeerId, u64, String),
        // LaunchUri { peer: PeerId, sid: u64, uri: String },
        AppControl {
            peer: PeerId,
            sid: u64,
            ctl: ControlMessage,
        },
        AppControlUpdate {
            peer: PeerId,
            status: ControlStatus,
        },
        // PeerCtlWaiting(PeerId),
        // PeerCtlSuccess(PeerId),
        // PeerCtlCancel(PeerId),
        // PeerCtlFailed(PeerId),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ControlMessage {
        LaunchUri { uri: String, ask: bool },
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ControlStatus {
        Waiting,
        Success,
        Cancelled,
        Failed,
    }
}

#[cfg(test)]
mod test {
    use p2p::peer::{PeerId, PeerMetadata};

    use crate::api::{
        cmd,
        event::{self, ControlStatus},
        query,
    };

    #[test]
    pub fn json() {
        println!(
            "{}",
            // serde_json::to_string(&event::CoreEvent::Discovered(PeerMetadata::default())).unwrap()
            serde_json::to_string(&event::CoreEvent::AppControlUpdate {
                peer: PeerId::default(),
                status: ControlStatus::Success
            })
            .unwrap()
        );
    }
}
