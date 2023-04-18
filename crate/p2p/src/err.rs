use thiserror::Error;

/// Errors while initializing P2p
#[derive(Debug, Error)]
pub enum InitError {
    /// The multicast IP is not a true multicast IP
    #[error("The address for discovery is not a multicast address")]
    NotMulticast,

    /// An unspecified network error occured
    #[error("A network related error occured")]
    Net(#[from] std::io::Error)
}

/// An error that can occur during the handshake process
#[derive(Debug,Error)]
pub enum HandshakeError {
    /// The parser failed
    #[error("The parser or i/o layer recieved an error")]
    Parse(#[from] ParseError),

    /// The remote peer reported an error
    #[error("An unspecific protocol error occured")]
    Failure(u32),

    /// The remote peer timed out
    #[error("The remote peer timed out")]
    Timeout,

    /// The remote peer unexpectedly disconnected
    #[error("The remote peer closed the connection")]
    Disconnect,

    /// The local peer could not authenticate the remote peer
    #[error("There was an authentication error")]
    Auth,

    /// The local peer unexpectedly recieved the wrong message
    #[error("The local peer received the wrong message")]
    Msg,

    /// The remote peer is unknown
    #[error("The peer was not found")]
    NotFound,

    /// The local peer is already connected
    #[error("A connection already exists")]
    Dup,

    /// The remote peer had no connectable addresses
    #[error("No connectable addresses")]
    Addr
}

impl From<ring::error::Unspecified> for HandshakeError {
    fn from(_: ring::error::Unspecified) -> Self {
        HandshakeError::Auth
    }
}

/// Represents an error that can occur when creating a [PeerId] from a string.
#[derive(Error, Debug)]
pub enum IdError {
    /// The id is too long
	#[error("the id must be 40 chars in length")]
	Length,

    /// The id can only contain alphanumeric character
	#[error("the id must be alphanumeric")]
	InvalidCharacters,
}

/// An error originating from parsing protocol packets
#[derive(Error, Debug)]
pub enum ParseError {
    /// The packet does not start with a proper signature
    #[error("This is not a protocol packet")]
    NotAPacket,

    /// The packet had an unexpected message type
    #[error("The unexpected message type {0:?} was found")]
    MsgType(crate::proto::MessageType),

    /// There was a problem performing an I/O operation
    #[error("The I/O operation failed")]
    IOError(#[from] std::io::Error),

    /// The byte could not be made into an enum value
    #[error("The value {0} is not a valid enum")]
    Enum(usize),

    /// The socket address is incorrectly formatted
    #[error("The value {0} is not a valid SocketAddr")]
    Addr(#[from] std::net::AddrParseError),

    /// The peer id is not valid
    #[error("The peer id {0} is not valid")]
    Id(#[from] IdError)
}

impl<T> From<num_enum::TryFromPrimitiveError<T>> for ParseError where T: num_enum::TryFromPrimitive, T::Primitive: Into<usize> {
    fn from(value: num_enum::TryFromPrimitiveError<T>) -> Self {
        ParseError::Enum(value.number.into())
    }
}