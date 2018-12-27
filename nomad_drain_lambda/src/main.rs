mod error;

use std::borrow::Cow;

use aws_lambda_events::event::autoscaling::AutoScalingEvent as Event;
use lambda_runtime::{error::HandlerError, lambda, Context};
use serde::{Deserialize, Serialize};

use nomad_drain::nomad::Client as NomadClient;
use nomad_drain::vault::Client as VaultClient;

use crate::error::Error;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct Config {
    /// Address of Nomad server
    #[serde(rename = "nomad_addr")]
    nomad_address: String,

    #[serde(default = "Config::default_use_nomad_token")]
    use_nomad_token: bool,

    /// Nomad token, if any
    nomad_token: Option<String>,

    #[serde(flatten)]
    vault_config: VaultConfig,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct VaultConfig {
    vault_token: Option<String>,

    #[serde(rename = "vault_addr")]
    vault_address: Option<String>,

    auth_path: Option<String>,
    auth_role: Option<String>,
    auth_header_value: Option<String>,

    nomad_path: Option<String>,
    nomad_role: Option<String>,
}

#[derive(Serialize)]
struct HandlerResult {
    pub message: String,
}

impl Config {
    /// Deserialize from the environment
    pub fn from_environment() -> Result<Self, Error> {
        Ok(envy::from_env()?)
    }

    pub fn new_nomad_client(&self) -> Result<NomadClient, Error> {
        let nomad_token = if self.use_nomad_token {
            Some(self.get_nomad_token()?)
        } else {
            None
        };

        let nomad_client = NomadClient::new(&self.nomad_address, nomad_token.as_ref(), None)?;

        Ok(nomad_client)
    }

    fn get_nomad_token(&self) -> Result<Cow<str>, Error> {
        match self.nomad_token {
            Some(ref token) => Ok(Cow::Borrowed(token.as_str())),
            None => {
                let vault_client = self.get_vault_client()?;

                let nomad_path = self
                    .vault_config
                    .nomad_path
                    .as_ref()
                    .ok_or_else(|| Error::MissingConfiguration("nomad_path".to_string()))?;
                let nomad_role = self
                    .vault_config
                    .nomad_path
                    .as_ref()
                    .ok_or_else(|| Error::MissingConfiguration("nomad_role".to_string()))?;

                Ok(Cow::Owned(
                    vault_client.get_nomad_token(nomad_path, nomad_role)?,
                ))
            }
        }
    }

    fn get_vault_client(&self) -> Result<VaultClient, Error> {
        let vault_address = self
            .vault_config
            .vault_address
            .as_ref()
            .ok_or_else(|| Error::MissingConfiguration("vault_address".to_string()))?;

        match self.vault_config.vault_token {
            Some(ref token) => Ok(VaultClient::new(vault_address, token, None)?),
            None => {
                let vault_auth_path = self
                    .vault_config
                    .auth_path
                    .as_ref()
                    .ok_or_else(|| Error::MissingConfiguration("auth_path".to_string()))?;
                let vault_auth_role = self
                    .vault_config
                    .auth_role
                    .as_ref()
                    .ok_or_else(|| Error::MissingConfiguration("auth_role".to_string()))?;

                let aws_credentials = nomad_drain::get_aws_credentials()?;

                Ok(nomad_drain::login_to_vault(
                    vault_address,
                    vault_auth_path,
                    vault_auth_role,
                    &aws_credentials,
                    self.vault_config
                        .auth_header_value
                        .as_ref()
                        .map(|s| s.as_str()),
                    None,
                )?)
            }
        }
    }

    const fn default_use_nomad_token() -> bool {
        true
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lambda!(lambda_wrapper);

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn lambda_wrapper(event: Event, context: Context) -> Result<HandlerResult, HandlerError> {
    match lambda_handler(&event, &context) {
        Ok(result) => Ok(result),
        Err(e) => Err(context.new_error(&e.to_string())),
    }
}

fn lambda_handler(event: &Event, context: &Context) -> Result<HandlerResult, Error> {
    let config = Config::from_environment()?;
    let nomad_client = config.new_nomad_client()?;

    Ok(HandlerResult {
        message: "Hello world".to_string(),
    })
}

// Environment deserialize https://github.com/softprops/envy
