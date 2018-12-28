use failure_derive::Fail;

/// Error type for this library
#[derive(Debug, Fail)]
pub enum Error {
    /// Errors related to retrieving AWS credentials
    #[fail(display = "Error retrieving AWS credentials: {}", _0)]
    CredentialsError(#[cause] rusoto_core::CredentialsError),
    /// Errors related to API HTTP calls
    #[fail(display = "Error making HTTP Request: {}", _0)]
    ReqwestError(#[cause] reqwest::Error),
    /// Errors parsing headers
    #[fail(display = "Error parsing HTTP header: {}", _0)]
    HeadersErrors(#[cause] reqwest::header::ToStrError),
    /// Errors related to URL parsing
    #[fail(display = "Error Parsing URL: {}", _0)]
    UrlParseError(#[cause] url::ParseError),
    /// Response from Vault was unexpected
    #[fail(display = "Unexpected response from Vault: {}", _0)]
    InvalidVaultResponse(String),
    /// Nomad Node not found
    #[fail(display = "No Nomad Node found for AWS instance ID: {}", instance_id)]
    NomadNodeNotFound { instance_id: String },
    /// Errors parsing Numbers
    #[fail(display = "Error parsing integer: {}", _0)]
    ParseIntError(#[cause] std::num::ParseIntError),
    /// Errors deserializing JSON
    #[fail(display = "Error deserializing JSON: {}", _0)]
    JsonError(#[cause] serde_json::Error),
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

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(error)
    }
}
