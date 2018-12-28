use failure_derive::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(
        display = "Error deserializing configuration from the environment: {}",
        _0
    )]
    ConfigurationDecodingError(#[cause] envy::Error),
    #[fail(display = "{}", _0)]
    LibError(#[cause] nomad_drain::Error),
    #[fail(display = "Error deserializing JSON: {}", _0)]
    JsonError(#[cause] serde_json::Error),
    #[fail(display = "Configuration option `{}` was expected but is missing", _0)]
    MissingConfiguration(String),
    #[fail(display = "Error completing ASG Lifecycle action: {}", _0)]
    AsgLifecycleError(#[cause] rusoto_autoscaling::CompleteLifecycleActionError),
    #[fail(display = "Expecting an Instance Terminating event, but got something else instead")]
    UnexpectedLifecycleTransition,
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
