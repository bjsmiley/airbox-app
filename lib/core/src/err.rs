use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("A configuration file error occured: {0}")]
    Conf(#[from] ConfError),

    #[error("An I/O error occured: {0}")]
    IO(#[from] std::io::Error),

    #[error("No local area ips found")]
    NoNetworkAccess,

    // #[error("An error occured initializing p2p: {0}")]
    // P2pInit(#[from] p2p::err::InitError),
    #[error("A p2p connection error occured: {0}")]
    P2pHandshake(#[from] p2p::err::ConnError),

    #[error("A json serialization error occured: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("A protocol error occured: {0}")]
    Protocol(String),

    #[error("A pairing error occured: {0}")]
    Pairing(#[from] p2p::err::PairingError),

    #[error("A qr code error occured: {0}")]
    QrCode(#[from] qrcode::types::QrError),

    #[error("A utf8 error occured: {0}")]
    Uft8(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, Error)]
pub enum ConfError {
    #[error("The I/O operation failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to read/write json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to access secret: {0}")]
    Secret(#[from] keyring::error::Error),
}

/// An error originating from parsing protocol packets
#[derive(Error, Debug)]
pub enum ParseError {
    /// There was a problem performing an I/O operation
    #[error("The I/O operation failed: {0}")]
    IOError(#[from] std::io::Error),

    /// The message was too large or the delimeter was not found
    #[error("Failed to seek to next message: {0}")]
    Delimiter(#[from] tokio_util::codec::LinesCodecError),

    /// The message contained invalid json
    #[error("Failed to convert next message to json: {0}")]
    Json(#[from] serde_json::Error),
}
