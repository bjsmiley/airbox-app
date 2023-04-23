use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    /// A Store error occured
    //#[error("A database operation failed")]
    //Store(#[from] rusqlite::Error),
    #[error("A configuration file error occured")]
    Conf(#[from] ConfError),
}

#[derive(Debug, Error)]
pub enum ConfError {
    #[error("Failed to read/write file")]
    IO(#[from] std::io::Error),
    #[error("Failed to read/write json")]
    Json(#[from] serde_json::Error),
    #[error("Failed to access secret")]
    Secret(#[from] keyring::error::Error),
}
