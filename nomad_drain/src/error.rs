use std::error;
use std::fmt;

/// Error type for this library
#[derive(Debug)]
pub enum Error {
    /// Errors related to retrieving AWS credentials
    CredentialsError(rusoto_core::CredentialsError),
    /// Errors related to API HTTP calls
    ReqwestError(reqwest::Error),
    /// Errors parsing headers
    HeadersErrors(reqwest::header::ToStrError),
    /// Errors related to URL parsing
    UrlParseError(url::ParseError),
    /// Response from Vault was unexpected
    InvalidVaultResponse(String),
    /// Nomad Node not found
    NomadNodeNotFound { instance_id: String },
    /// Errors parsing Numbers
    ParseIntError(std::num::ParseIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CredentialsError(ref inner) => {
                write!(f, "Error retrieving AWS credentials: {}", inner)
            }
            Error::ReqwestError(ref inner) => inner.fmt(f),
            Error::HeadersErrors(ref inner) => inner.fmt(f),
            Error::UrlParseError(ref inner) => inner.fmt(f),
            Error::InvalidVaultResponse(ref reason) => {
                write!(f, "Response from Vault was unexpected: {}", reason)
            }
            Error::NomadNodeNotFound { ref instance_id } => write!(
                f,
                "Unable to find the nomad node with instance ID: {}",
                instance_id
            ),
            Error::ParseIntError(ref inner) => inner.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::CredentialsError(ref inner) => Some(inner),
            Error::ReqwestError(ref inner) => Some(inner),
            Error::HeadersErrors(ref inner) => Some(inner),
            Error::UrlParseError(ref inner) => Some(inner),
            Error::ParseIntError(ref inner) => Some(inner),
            Error::InvalidVaultResponse(_) | Error::NomadNodeNotFound { .. } => None,
        }
    }
}

impl From<rusoto_core::CredentialsError> for Error {
    fn from(error: rusoto_core::CredentialsError) -> Self {
        Error::CredentialsError(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error)
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(error: reqwest::header::ToStrError) -> Self {
        Error::HeadersErrors(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::UrlParseError(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::ParseIntError(error)
    }
}
