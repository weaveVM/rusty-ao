use base64::DecodeError;
use bundlr_sdk::error::BundlrError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum AoErrors {
    #[error("Base 64 is not recognized")]
    Base64ReadingError,

    #[error("The instance of bundlr generated an error")]
    BundlrError,

    #[error("Invalid MU API Url")]
    InvalidMUApiUrl,

    #[error("The signer could not be created")]
    ErrorConstructingSigner,

    #[error("Signer is invalid")]
    InvalidSigner,

    #[error("Transaction is either invalid or broken")]
    InvalidTransaction,

    #[error("The server did not respond as expected")]
    InvalidServerResponse,

    #[error("The server did not respond as expected")]
    InvalidResponseDeserialization,
}
