use std::collections::{HashSet};

use crate::err::ConfError;
use p2p::peer::{self, Identity};

pub static SERVICE_NAME: &str = "flydrop";
pub static IDENTITY: &str = "Identity";
pub static TOTP_AUTH: &str = "_Totp";

/// Get or create a new identity
pub(crate) fn get_identity() -> Result<peer::Identity, ConfError> {
    let e = keyring::Entry::new(SERVICE_NAME, IDENTITY)?;
    match e.get_password() {
        Ok(data) => Ok(serde_json::from_str(&data)?),
        Err(keyring::error::Error::NoEntry) => {
            let id = Identity::new();
            let data = serde_json::to_string(&id)?;
            e.set_password(&data)?;
            Ok(id)
        }
        Err(x) => Err(ConfError::Secret(x)),
    }
}

pub(crate) fn get_totp(peer: &peer::PeerId) -> Result<String, ConfError> {
    let key = peer.inner().clone() + TOTP_AUTH;
    let e = keyring::Entry::new(SERVICE_NAME, &key)?;
    Ok(e.get_password()?)
}

pub(crate) fn to_known(peers: &HashSet<peer::PeerMetadata>) -> Vec<peer::PeerCandidate> {
    let mut map = Vec::new();
    for peer in peers {
        if let Ok(pwd) = get_totp(&peer.id) {
            if let Ok(auth) = p2p::pairing::PairingAuthenticator::new(pwd.into_bytes()) {
                map.push(peer::PeerCandidate::new(peer, auth));
            }
        }
    }
    map
}

/// used for testing, to mock the underlying secret store
pub fn mock_store() {
    use keyring::{mock::default_credential_builder, set_default_credential_builder};
    set_default_credential_builder(default_credential_builder());
}
