use ring::{hmac, error};


pub(crate) fn sign(key: &[u8], data: &[u8]) -> hmac::Tag {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::sign(&key, data)
}

pub(crate) fn verify(key: &[u8], data: &[u8], hmac: &[u8]) -> Result<(),error::Unspecified> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::verify(&key, data, hmac)
}


#[cfg(test)]
mod tests {

    #[test]
    fn hmac_peer_id_auth_code() -> Result<(), Box<dyn std::error::Error>> {
        let secret = String::from("SuperSecureSecret");
        let id = String::from("0123456789012345678901234567890123456789");
        let pid = crate::peer::PeerId::from_string(id)?;
        let peer = pid.as_bytes();
        let totp = totp_rs::Secret::Encoded(secret).to_bytes().unwrap();
        let auth = crate::pairing::PairingAuthenticator::new(totp)?;
        let code = auth.generate()?;
        let tag = super::sign(code.as_bytes(), peer);
        assert_eq!((), super::verify(code.as_bytes(), peer, tag.as_ref()).unwrap());
        Ok(())
    }
}