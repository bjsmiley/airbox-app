use std::{str::FromStr, time::SystemTimeError};

use thiserror::Error;
use totp_rs::{TotpUrlError, TOTP, Secret};


pub struct Png(String);

#[derive(Debug)]
pub struct PairingAuthenticator {
    totp: TOTP
}

impl PairingAuthenticator {
    pub fn new(secret: Vec<u8>) -> Result<Self, PairingError> {
        Ok(Self { 
            totp: TOTP::new(
                totp_rs::Algorithm::SHA256,
                8,
                1,
                15,
                secret,
                None,
                "airbox-client".to_string())?})
    }

    pub fn from_url<S: AsRef<str>>(url: S) -> Result<Self, PairingError> {
        Ok(Self { totp: TOTP::from_url(url)? })
    }

    pub fn to_qr_code(&self) -> Result<Png, PairingError> {
        let png = self.totp.get_qr().map_err(|e| PairingError::QrCode(e))?;
        Ok(Png(png))
    }

    pub fn check(&self, token: &str) -> Result<bool, PairingError> {
        Ok(self.totp.check_current(token)?)
    }

    pub fn generate(&self) -> Result<String, PairingError> {
        Ok(self.totp.generate_current()?)
    }
}

impl ToString for PairingAuthenticator {
    fn to_string(&self) -> String {
        self.totp.get_secret_base32()
    }
}

impl FromStr for PairingAuthenticator {
    type Err = PairingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let secret_b32 = Secret::Encoded(String::from(s));
        Self::new(secret_b32.to_bytes().map_err(|e| PairingError::Secret(format!("{:?}", e)))?)
    }
}



#[derive(Error, Debug)]
pub enum PairingError {
    #[error("Error initializing Totp")]
    Totp(#[from] TotpUrlError),
    #[error("Error generating QR code")]
    QrCode(String),
    #[error("Error parsing secret")]
    Secret(String),
    #[error("Errors checking system time")]
    CheckError(#[from] SystemTimeError)
}