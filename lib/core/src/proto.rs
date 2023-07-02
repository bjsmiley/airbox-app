use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder, LinesCodec};

/// CTL message could not make it to presentation layer
pub const CTL_UNKNOWN_ERR: u32 = 1;
/// CTL message was declined by user
pub const CTL_CANCEL: u32 = 2;

/// These messages are sent across during an active session between two connected and authenticated devices.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: u64,
    pub ctl: Ctl,
}

/// These messages are sent across during an active session between two connected and authenticated devices.
// #[derive(Debug, Serialize, Deserialize)]
// pub enum Session {
//     Ctl(Ctl),
// }

/// Application control messages
#[derive(Debug, Serialize, Deserialize)]
pub enum Ctl {
    Request(CtlRequest),
    Response(CtlResponse), // /// Request to launch a uri on the host device
                           // LaunchUri(String),
                           // /// Response from the host device to launch a uri
                           // LaunchUriResult(LaunchUriResult),
}

/// The request to send to a host device
#[derive(Debug, Serialize, Deserialize)]
pub enum CtlRequest {
    /// Request to launch a uri on the host device
    LaunchUri(String),
    // Introduce self
    // Hello,
}

/// The response from attempting to perform an app control requet on a host
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

pub struct SessionCodec(LinesCodec);

impl Default for SessionCodec {
    fn default() -> Self {
        Self(Default::default())
    }
}

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

// pub struct RequestCodec(LinesCodec);

// impl Default for RequestCodec {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }

// impl Encoder<PeerRequest> for RequestCodec {
//     type Error = crate::err::ParseError;

//     fn encode(&mut self, item: PeerRequest, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
//         let msg = serde_json::to_string(&item)?;
//         Ok(self.0.encode(msg, dst)?)
//     }
// }

// impl Decoder for RequestCodec {
//     type Item = PeerRequest;

//     type Error = crate::err::ParseError;

//     fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//         match self.0.decode(src)? {
//             Some(json) => Ok(Some(serde_json::from_slice::<Self::Item>(json.as_bytes())?)),
//             None => Ok(None),
//         }
//     }
// }

// pub struct ReponseCodec(LinesCodec);

// impl Default for ReponseCodec {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }

// impl Encoder<PeerResponse> for ReponseCodec {
//     type Error = crate::err::ParseError;

//     fn encode(&mut self, item: PeerResponse, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
//         let msg = serde_json::to_string(&item)?;
//         Ok(self.0.encode(msg, dst)?)
//     }
// }

// impl Decoder for ReponseCodec {
//     type Item = PeerResponse;

//     type Error = crate::err::ParseError;

//     fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//         match self.0.decode(src)? {
//             Some(json) => Ok(Some(serde_json::from_slice::<Self::Item>(json.as_bytes())?)),
//             None => Ok(None),
//         }
//     }
// }
