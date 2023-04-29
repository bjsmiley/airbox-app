use std::str::FromStr;

use totp_rs::{Secret, TOTP};

use crate::err;

pub struct Png(String);

#[derive(Debug, Clone)]
pub struct PairingAuthenticator {
    totp: TOTP,
}

impl PairingAuthenticator {
    pub fn new(secret: Vec<u8>) -> Result<Self, err::PairingError> {
        Ok(Self {
            totp: TOTP::new(
                totp_rs::Algorithm::SHA256,
                8,
                1,
                15,
                secret,
                None,
                "flydrop-client".to_string(),
            )?,
        })
    }

    pub fn from_url<S: AsRef<str>>(url: S) -> Result<Self, err::PairingError> {
        Ok(Self {
            totp: TOTP::from_url(url)?,
        })
    }

    pub fn to_qr_code(&self) -> Result<Png, err::PairingError> {
        let png = self.totp.get_qr().map_err(err::PairingError::QrCode)?;
        Ok(Png(png))
    }

    pub fn check(&self, token: &str) -> Result<bool, err::PairingError> {
        Ok(self.totp.check_current(token)?)
    }

    pub fn generate(&self) -> Result<String, err::PairingError> {
        Ok(self.totp.generate_current()?)
    }
}

impl ToString for PairingAuthenticator {
    fn to_string(&self) -> String {
        self.totp.get_secret_base32()
    }
}

impl FromStr for PairingAuthenticator {
    type Err = err::PairingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let secret_b32 = Secret::Encoded(String::from(s));
        Self::new(
            secret_b32
                .to_bytes()
                .map_err(|e| Self::Err::Secret(format!("{:?}", e)))?,
        )
    }
}
