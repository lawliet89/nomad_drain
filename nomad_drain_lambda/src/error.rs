use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ConfigurationDecodingError(envy::Error),
    LibError(nomad_drain::Error),
    MissingConfiguration(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ConfigurationDecodingError(ref inner) => inner.fmt(f),
            Error::LibError(ref inner) => inner.fmt(f),
            Error::MissingConfiguration(ref field) => write!(
                f,
                "Configuration `{}` is required but was not provided",
                field
            ),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::ConfigurationDecodingError(ref inner) => Some(inner),
            Error::LibError(ref inner) => Some(inner),
            Error::MissingConfiguration(_) => None,
        }
    }
}

impl From<envy::Error> for Error {
    fn from(error: envy::Error) -> Self {
        Error::ConfigurationDecodingError(error)
    }
}

impl From<nomad_drain::Error> for Error {
    fn from(error: nomad_drain::Error) -> Error {
        Error::LibError(error)
    }
}
