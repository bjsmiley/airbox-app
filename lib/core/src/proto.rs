use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder, LinesCodec};

/// CTL message could not make it to presentation layer
pub const CTL_UNKNOWN_ERR: u32 = 1;
/// CTL message was declined by user
pub const CTL_CANCEL: u32 = 2;

/// These messages are sent across during an active session between two connected and authenticated devices.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// The session id this message is associated with
    pub id: u64,
    /// The app control message body
    pub ctl: Ctl,
}

/// Application control messages
#[derive(Debug, Serialize, Deserialize)]
pub enum Ctl {
    /// An app control request sent to a remote peer
    Request(CtlRequest),
    /// A response to an app control request sent from a remote peer
    Response(CtlResponse),
}

/// The request to send to a remote peer
#[derive(Debug, Serialize, Deserialize)]
pub enum CtlRequest {
    /// Request to launch a uri on the host device
    LaunchUri(String),
}

/// The response from attempting to perform an app control request on a host
#[derive(Debug, Serialize, Deserialize)]
pub enum CtlResponse {
    /// The host device successfully completed the app control request
    Success,
    /// The host device is awaiting user input
    Waiting,
    /// The host device failed to complete the app control request
    Error(u32),
    /// The host device declined to complete the app control request
    Cancel,
}

#[derive(Default)]
pub struct SessionCodec(LinesCodec);

impl Encoder<Session> for SessionCodec {
    type Error = crate::err::ParseError;

    fn encode(&mut self, item: Session, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let msg = serde_json::to_string(&item)?;
        Ok(self.0.encode(msg, dst)?)
    }
}

impl Decoder for SessionCodec {
    type Item = Session;

    type Error = crate::err::ParseError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(json) => Ok(Some(serde_json::from_slice::<Self::Item>(json.as_bytes())?)),
            None => Ok(None),
        }
    }
}
