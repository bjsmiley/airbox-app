use std::collections::HashSet;

use crate::err::CoreError;
use base64::Engine;
use p2p::peer;

pub static SVC: &str = "app://flydrop.io";
pub static ID: &str = "Identity";
pub static CERT: &str = "Certificate";
pub static PK: &str = "PrivateKey";
pub static TOTP: &str = "_Totp";

pub(crate) struct SecretStore {
    suf: String,
}

impl SecretStore {
    pub fn new(suf: String) -> SecretStore {
        SecretStore { suf }
    }

    pub fn get<'a, T>(&self, key: &str) -> Result<Option<T>, CoreError>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.get_entry(key)?.get_password() {
            Ok(val) => Ok(serde_json::from_str(val.as_str())?),
            Err(keyring::error::Error::NoEntry) => Ok(None),
            Err(err) => Err(CoreError::Secret(err.to_string())),
        }
    }

    pub fn set<T>(&self, key: &str, val: &T) -> Result<(), CoreError>
    where
        T: serde::Serialize,
    {
        self.get_entry(key)?
            .set_password(serde_json::to_string(val)?.as_str())
            .map_err(|err| CoreError::Secret(err.to_string()))
    }

    pub fn put<'a, T>(&self, key: &str, op: impl FnOnce() -> T) -> Result<T, CoreError>
    where
        T: serde::de::DeserializeOwned,
        T: serde::Serialize,
    {
        match self.get(key)? {
            None => {
                let val = op();
                self.set(key, &val)?;
                Ok(val)
            }
            Some(val) => Ok(val),
        }
    }

    fn get_entry(&self, key: &str) -> Result<keyring::Entry, CoreError> {
        keyring::Entry::new(SVC, format!("{}_{}", key, self.suf).as_str())
            .map_err(|err| CoreError::Secret(err.to_string()))
    }
}

impl SecretStore {
    /// Get or create a new identity
    pub(crate) fn get_identity(&self) -> Result<peer::Identity, CoreError> {
        let (cert, key) = match (self.get::<String>(CERT), self.get::<String>(PK)) {
            (Ok(Some(cert64)), Ok(Some(key64))) => {
                let cert = base64::engine::general_purpose::STANDARD_NO_PAD
                    .decode(cert64)
                    .map_err(|e| CoreError::Base64(e.to_string()))?;
                let key = base64::engine::general_purpose::STANDARD_NO_PAD
                    .decode(key64)
                    .map_err(|e| CoreError::Base64(e.to_string()))?;
                (cert, key)
            }
            _ => {
                let (cert, key) = peer::Identity::default().to_raw();
                self.set(CERT, &cert)?;
                self.set(PK, &key)?;
                (cert, key)
            }
        };

        Ok(peer::Identity::from_raw(cert, key))
    }

    pub(crate) fn get_totp(&self, peer: &peer::PeerId) -> Result<Option<String>, CoreError> {
        let key = peer.inner().clone() + TOTP;
        self.get(&key)
    }

    pub(crate) fn set_totp(&self, peer: &peer::PeerId, totp: &String) -> Result<(), CoreError> {
        let key = peer.inner().clone() + TOTP;
        self.set(&key, &totp)
    }

    pub(crate) fn to_known(&self, peers: &HashSet<peer::PeerMetadata>) -> Vec<peer::PeerCandidate> {
        let mut map = Vec::new();
        for peer in peers {
            if let Ok(Some(pwd)) = self.get_totp(&peer.id) {
                if let Ok(auth) = p2p::pairing::PairingAuthenticator::new(pwd.into_bytes()) {
                    map.push(peer::PeerCandidate::new(peer, auth));
                }
            }
        }
        map
    }
}

// /// Get or create a new identity
// pub(crate) fn get_identity() -> Result<peer::Identity, ConfError> {
//     let e = keyring::Entry::new(SVC, PK)?;
//     let pk = match e.get_password() {
//         Ok(data) => Ok(serde_json::from_str(&data)?),
//         Err(keyring::error::Error::NoEntry) => {
//             let id = Identity::default();
//             let (cert, pk) = id.into_rustls();
//             let data = pk.;
//             e.set_password(&data)?;
//             // let data = serde_json::to_string(&id)?;
//             // e.set_password(&data)?;
//             // Ok(id)
//         }
//         Err(x) => Err(ConfError::Secret(x)),
//     }
// }

// pub(crate) fn get_totp(peer: &peer::PeerId) -> Result<String, ConfError> {
//     let key = peer.inner().clone() + TOTP;
//     let e = keyring::Entry::new(SVC, &key)?;
//     Ok(e.get_password()?)
// }

// pub(crate) fn set_totp(peer: &peer::PeerId, secret: String) -> Result<(), ConfError> {
//     let key = peer.inner().clone() + TOTP;
//     let e = keyring::Entry::new(SVC, &key)?;
//     e.set_password(secret.as_str())?;
//     Ok(())
// }

// pub(crate) fn to_known(peers: &HashSet<peer::PeerMetadata>) -> Vec<peer::PeerCandidate> {
//     let mut map = Vec::new();
//     for peer in peers {
//         if let Ok(pwd) = get_totp(&peer.id) {
//             if let Ok(auth) = p2p::pairing::PairingAuthenticator::new(pwd.into_bytes()) {
//                 map.push(peer::PeerCandidate::new(peer, auth));
//             }
//         }
//     }
//     map
// }

/// used for testing, to mock the underlying secret store
pub fn mock_store() {
    use keyring::{mock::default_credential_builder, set_default_credential_builder};
    set_default_credential_builder(default_credential_builder());
}
