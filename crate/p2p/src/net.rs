use std::{sync::Arc, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, time::timeout};
use tokio_util::codec::Framed;
use tracing::{debug, error};

use crate::{err, peer::{Peer, PeerCandidate}, proto::{ConnectCodec, Connect}, crypto::{hmac_encrypt, hmac_verify}, pairing::PairingAuthenticator, manager::P2pManager};

const TIMEOUT_ERR: u32 = 2001;
const NOT_FOUND_ERR: u32 = 2002;
const AUTH_ERR: u32 = 2003;



/// handshake as the client to attempt to connect as a connected peer
pub(crate) async fn connect(manager: &Arc<P2pManager>, conn: TcpStream, peer: &PeerCandidate) -> Result<Peer, err::HandshakeError> {
    // get auth ready
    let auth = PairingAuthenticator::new(peer.auth_secret.clone().into_bytes()).unwrap();
    let code = auth.generate().unwrap();
    let key = code.as_bytes();
    let mut frame = Framed::new(conn, ConnectCodec);
    
    let encrypted_id = hmac_encrypt(key, manager.id.as_bytes());
    frame.send(Connect::ConnectRequest{ id: manager.id.clone(), tag: encrypted_id.as_ref().to_vec()}).await?;

    let Ok(response) = timeout(Duration::from_secs(1), frame.next()).await else {
        error!("peer timed out waiting for ConnectResponse");
        frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
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
                    if let Err(e) = hmac_verify(key, peer.id.as_bytes(), &tag){
                        error!("Error verifying totp hmac: {:?}", e);
                        frame.send(crate::proto::Connect::ConnectionFailure(AUTH_ERR)).await;
                        return Err(err::HandshakeError::Auth);
                    }
                    frame.send(Connect::ConnectionCompleteRequest).await?;
                    let Ok(complete) = timeout(Duration::from_secs(1), frame.next()).await else {
                        error!("peer timed out waiting for ConnectionCompleteResponse");
                        frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
                        return Err(err::HandshakeError::Timeout);
                    };
                    match complete {
                        Some(res) => {
                            match res? {
                                Connect::ConnectionCompleteResponse => {
                                    let connected = Peer::new(manager, crate::peer::ConnectionType::Client,
                                         frame.into_inner(), peer.metadata.clone()).unwrap();
                                    //self.connected_peers.insert(connected.id.clone(), connected);
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

    // get auth ready


    let mut frame = Framed::new(conn, ConnectCodec);

    // timeout in 1 sec to ensure no bad intent 
    let Ok(request) = timeout(Duration::from_secs(1), frame.next()).await else {
        error!("peer timed out waiting for ConnectionRequest");
        frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
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
                        frame.send(crate::proto::Connect::ConnectionFailure(NOT_FOUND_ERR)).await;
                        error!("peer is not known nor discovered");
                        return Err(err::HandshakeError::NotFound);
                    };
                    debug!("validating peer's totp code");
                    let auth = PairingAuthenticator::new(peer.auth_secret.into_bytes()).unwrap();
                    let code = auth.generate().unwrap();
                    let key = code.as_bytes();
                    if let Err(e) = hmac_verify(key, peer.id.as_bytes(), &tag){
                        error!("Error verifying totp hmac: {:?}", e);
                        frame.send(crate::proto::Connect::ConnectionFailure(AUTH_ERR)).await;
                        return Err(err::HandshakeError::Auth);
                    }
                    let encrpyted_id = hmac_encrypt(key, manager.id.as_bytes());
                    frame.send(crate::proto::Connect::ConnectionResponse(encrpyted_id.as_ref().to_vec())).await?;

                    let Ok(complete) = timeout(Duration::from_secs(1), frame.next()).await else {
                        error!("peer timed out waiting for ConnectionCompleteRequest");
                        frame.send(crate::proto::Connect::ConnectionFailure(TIMEOUT_ERR)).await;
                        return Err(err::HandshakeError::Timeout);
                    };
                    match complete {
                        Some(res) => {
                            match res? {
                                Connect::ConnectionCompleteRequest => {
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