use ring::{hmac, error};


pub(crate) fn hmac_encrypt(key: &[u8], data: &[u8]) -> hmac::Tag {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::sign(&key, data)
}

pub(crate) fn hmac_verify(key: &[u8], data: &[u8], hmac: &[u8]) -> Result<(),error::Unspecified> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::verify(&key, data, hmac)
}


#[cfg(test)]
mod tests {

    use super::{hmac_encrypt, hmac_verify};


    #[test]
    fn hmac_peer_id_auth_code() {
        let secret = totp_rs::Secret::Encoded("OBWGC2LOFVZXI4TJNZTS243FMNZGK5BNGEZDG".to_string()).to_bytes().unwrap();
        let auth = crate::pairing::PairingAuthenticator::new(secret).unwrap();
        let peer_id = crate::peer::PeerId::from_string(String::from("0123456789012345678901234567890123456789")).unwrap();
        let code = auth.generate().unwrap();
        let tag = hmac_encrypt(code.as_bytes(), peer_id.as_bytes());
        // println!("{}", std::str::from_utf16(tag.as_ref()).unwrap());
        //let eng = base64::engine::general_purpose::URL_SAFE;
        // println!("{}", eng.encode(tag.as_ref()));
        //println!("{}", String::from_utf8_lossy(tag.as_ref()));
        // println!("{}", tag.as_ref().len());
        assert_eq!((), hmac_verify(code.as_bytes(), peer_id.as_bytes(), tag.as_ref()).unwrap());
    }
}