use std::error;
use std::fmt;

/// Error type for this library
#[derive(Debug)]
pub enum Error {
    CredentialsError(rusoto_core::CredentialsError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CredentialsError(ref inner) => {
                write!(f, "Error retrieving AWS credentials: {}", inner)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::CredentialsError(ref inner) => Some(inner),
        }
    }
}

impl From<rusoto_core::CredentialsError> for Error {
    fn from(error: rusoto_core::CredentialsError) -> Self {
        Error::CredentialsError(error)
    }
}
