use std::error;
use std::fmt;

/// Error type for this library
#[derive(Debug)]
pub enum Error {
    /// Errors related to retrieving AWS credentials
    CredentialsError(rusoto_core::CredentialsError),
    /// Errors related to API HTTP calls
    ReqwestError(reqwest::Error),
    /// Errors related to URL parsing
    UrlParseError(url::ParseError),
    /// Response from Vault was unexpected
    InvalidVaultResponse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CredentialsError(ref inner) => {
                write!(f, "Error retrieving AWS credentials: {}", inner)
            }
            Error::ReqwestError(ref inner) => inner.fmt(f),
            Error::UrlParseError(ref inner) => inner.fmt(f),
            Error::InvalidVaultResponse(ref reason) => {
                write!(f, "Response from Vault was unexpected: {}", reason)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::CredentialsError(ref inner) => Some(inner),
            Error::ReqwestError(ref inner) => Some(inner),
            Error::UrlParseError(ref inner) => Some(inner),
            Error::InvalidVaultResponse(_) => None,
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

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::UrlParseError(error)
    }
}
