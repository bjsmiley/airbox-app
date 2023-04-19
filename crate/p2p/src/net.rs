use std::{sync::Arc, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, time::timeout};
use tokio_util::codec::Framed;
use tracing::{debug, error};

use crate::{err, peer::{Peer, PeerCandidate}, proto::{ConnectCodec, Connect}, hmac, pairing::PairingAuthenticator, manager::P2pManager};

const TIMEOUT_ERR: u32 = 2001;
const NOT_FOUND_ERR: u32 = 2002;
const AUTH_ERR: u32 = 2003;



/// handshake as the client to attempt to connect as a connected peer
pub(crate) async fn connect(manager: &Arc<P2pManager>, conn: TcpStream, peer: &PeerCandidate) -> Result<Peer, err::HandshakeError> {
    // get auth code
    let code = peer.auth.generate().unwrap();
    let key = code.as_bytes();
    let tag = hmac::sign(key, manager.id.as_bytes());

    // send a connect request
    let mut frame = Framed::new(conn, ConnectCodec);
    frame.send(Connect::ConnectRequest{ id: manager.id.clone(), tag: tag.as_ref().to_vec()}).await?;

    // wait for a connect response
    let Ok(response) = timeout(Duration::from_secs(1), frame.next()).await else {
        error!("peer timed out waiting for ConnectResponse");
        _ = frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
        return Err(err::HandshakeError::Timeout);
    };
    match response {
        None => {
            error!("peer closed the connection");
            return Err(err::HandshakeError::Disconnect);
        }
        Some(res) => {
            match res? {
                Connect::ConnectionResponse(tag) => {
                    debug!("validating peer's totp code");
                    if let Err(e) = hmac::verify(key, peer.id.as_bytes(), &tag){
                        error!("Error verifying totp hmac: {:?}", e);
                        _ = frame.send(crate::proto::Connect::ConnectionFailure(AUTH_ERR)).await;
                        return Err(err::HandshakeError::Auth);
                    }
                    // send a complete request & wait for a complete response
                    frame.send(Connect::ConnectionCompleteRequest).await?;
                    let Ok(complete) = timeout(Duration::from_secs(1), frame.next()).await else {
                        error!("peer timed out waiting for ConnectionCompleteResponse");
                        _ = frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
                        return Err(err::HandshakeError::Timeout);
                    };
                    match complete {
                        Some(res) => {
                            match res? {
                                Connect::ConnectionCompleteResponse => {
                                    let connected = Peer::new(manager, crate::peer::ConnectionType::Client,
                                         frame.into_inner(), peer.metadata.clone()).unwrap();
                                    debug!("Peer is connected!");
                                    return Ok(connected);
                                }
                                _ => {
                                    error!("peer recieved the wrong message instead of ConnectionCompleteResponse");
                                    return Err(err::HandshakeError::Msg);
                                }
                            }
                        },
                        None => {
                            error!("peer closed the connection");
                            return Err(err::HandshakeError::Disconnect);
                        },
                    }
                },
                Connect::ConnectionFailure(code) => {
                    error!("received error {} instead of ConnectionResponse", code);
                    return Err(err::HandshakeError::Failure(code));
                }
                _ => {
                    error!("peer recieved the wrong message instead of ConnectionResponse");
                    return Err(err::HandshakeError::Msg);
                }
            }
        }
    }
}

/// handshake as the host to accept an incoming tcp connection as a connected peer
pub(crate) async fn accept(manager: &Arc<P2pManager>, conn: TcpStream) -> Result<Peer,err::HandshakeError> {

    let mut frame = Framed::new(conn, ConnectCodec);

    // timeout in 1 sec to ensure no bad intent 
    // wait for a connect request
    let Ok(request) = timeout(Duration::from_secs(1), frame.next()).await else {
        error!("peer timed out waiting for ConnectionRequest");
        _ = frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
        return Err(err::HandshakeError::Timeout);
    };
    match request {
        None => {
            error!("peer closed the connection");
            return Err(err::HandshakeError::Disconnect);
        },
        Some(req) => {
            match req? {
                Connect::ConnectRequest{ id, tag} => {
                    let Some(peer) = manager.get_peer_candidate(&id) else {
                        _ = frame.send(crate::proto::Connect::ConnectionFailure(NOT_FOUND_ERR)).await;
                        error!("peer is not known nor discovered");
                        return Err(err::HandshakeError::NotFound);
                    };
                    debug!("validating peer's totp code");
                    let code = peer.auth.generate().unwrap();
                    let key = code.as_bytes();
                    if let Err(e) = hmac::verify(key, peer.id.as_bytes(), &tag){
                        error!("Error verifying totp hmac: {:?}", e);
                        _ = frame.send(crate::proto::Connect::ConnectionFailure(AUTH_ERR)).await;
                        return Err(err::HandshakeError::Auth);
                    }
                    let tag = hmac::sign(key, manager.id.as_bytes());
                    // send a connect response & wait for a complete request
                    frame.send(crate::proto::Connect::ConnectionResponse(tag.as_ref().to_vec())).await?;
                    let Ok(complete) = timeout(Duration::from_secs(1), frame.next()).await else {
                        error!("peer timed out waiting for ConnectionCompleteRequest");
                        _ = frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
                        return Err(err::HandshakeError::Timeout);
                    };
                    match complete {
                        Some(res) => {
                            match res? {
                                Connect::ConnectionCompleteRequest => {
                                    // send a complete response
                                    frame.send(Connect::ConnectionCompleteResponse).await?;
                                    let connected = Peer::new(manager, crate::peer::ConnectionType::Server, 
                                     frame.into_inner(), peer.metadata).unwrap();
                                    debug!("Peer is connected!");
                                    return Ok(connected);
                                }
                                _ => {
                                    error!("peer recieved the wrong message instead of ConnectionCompleteRequest");
                                    return Err(err::HandshakeError::Msg);
                                }
                            }
                        }
                        None => {
                            error!("peer closed the connection");
                            return Err(err::HandshakeError::Disconnect);
                        },
                    }
                },
                Connect::ConnectionFailure(code) => {
                    error!("received error {} instead of ConnectionRequest", code);
                    return Err(err::HandshakeError::Failure(code));
                },
                _ => {
                    error!("peer recieved the wrong message instead of ConnectionRequest");
                    return Err(err::HandshakeError::Msg);
                }
            }
        }
    }
}