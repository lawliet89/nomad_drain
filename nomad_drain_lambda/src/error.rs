use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ConfigurationDecodingError(envy::Error),
    LibError(nomad_drain::Error),
    JsonError(serde_json::Error),
    MissingConfiguration(String),
    AsgLifecycleError(rusoto_autoscaling::CompleteLifecycleActionError),
    UnexpectedLifecycleTransition,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ConfigurationDecodingError(ref inner) => inner.fmt(f),
            Error::LibError(ref inner) => inner.fmt(f),
            Error::JsonError(ref inner) => inner.fmt(f),
            Error::AsgLifecycleError(ref inner) => inner.fmt(f),
            Error::MissingConfiguration(ref field) => write!(
                f,
                "Configuration `{}` is required but was not provided",
                field
            ),
            Error::UnexpectedLifecycleTransition => write!(f, "Expecting an Instance Terminating event, but lambda was fired with an Instance Launching Event instead")
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::ConfigurationDecodingError(ref inner) => Some(inner),
            Error::LibError(ref inner) => Some(inner),
            Error::JsonError(ref inner) => Some(inner),
            Error::AsgLifecycleError(ref inner) => Some(inner),
            Error::MissingConfiguration(_) | Error::UnexpectedLifecycleTransition => None,
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

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::JsonError(error)
    }
}

impl From<rusoto_autoscaling::CompleteLifecycleActionError> for Error {
    fn from(error: rusoto_autoscaling::CompleteLifecycleActionError) -> Self {
        Error::AsgLifecycleError(error)
    }
}
