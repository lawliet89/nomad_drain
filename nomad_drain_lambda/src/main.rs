mod error;

use std::borrow::Cow;

use aws_lambda_events::event::autoscaling::AutoScalingEvent as Event;
use failure::Fail;
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{error, info};
use rusoto_autoscaling::{Autoscaling, AutoscalingClient, CompleteLifecycleActionType};
use serde::{Deserialize, Serialize};

use nomad_drain::nomad::Client as NomadClient;
use nomad_drain::vault::Client as VaultClient;
use nomad_drain::Secret;

use crate::error::Error;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct Config {
    /// Address of Nomad server
    /// Deserialized from `NOMAD_ADDR`
    #[serde(rename = "nomad_addr")]
    nomad_address: String,

    /// Use Nomad Token or not
    #[serde(default = "Config::default_use_nomad_token")]
    use_nomad_token: bool,

    /// Nomad token, if any
    nomad_token: Option<Secret>,

    #[serde(flatten)]
    vault_config: VaultConfig,
    // Implicitly: RUST_LOG via `env_logger.
    // See https://docs.rs/env_logger/0.6.0/env_logger/#enabling-logging
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct VaultConfig {
    vault_token: Option<Secret>,

    #[serde(rename = "vault_addr")]
    vault_address: Option<String>,

    auth_path: Option<String>,
    auth_role: Option<String>,
    auth_header_value: Option<String>,

    nomad_path: Option<String>,
    nomad_role: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct AsgEventDetails {
    pub lifecycle_action_token: String,
    pub auto_scaling_group_name: String,
    #[serde(rename = "EC2InstanceId")]
    pub instance_id: String,
    pub lifecycle_transition: AsgLifecycleTransition,
    pub lifecycle_hook_name: String,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
enum AsgLifecycleTransition {
    #[serde(rename = "autoscaling:EC2_INSTANCE_LAUNCHING")]
    InstanceLaunching,
    #[serde(rename = "autoscaling:EC2_INSTANCE_TERMINATING")]
    InstanceTerminating,
}

#[derive(Serialize)]
struct HandlerResult {
    pub instance_id: String,
    pub node_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Config {
    /// Deserialize from the environment
    pub fn from_environment() -> Result<Self, Error> {
        Ok(envy::from_env()?)
    }

    pub fn new_nomad_client(&self) -> Result<NomadClient, Error> {
        info!("Building Nomad Client");
        let nomad_token = if self.use_nomad_token {
            info!("Using Nomad token");
            Some(self.get_nomad_token()?)
        } else {
            info!("No Nomad token in use");
            None
        };

        let nomad_client = NomadClient::new(&self.nomad_address, nomad_token.as_ref(), None)?;

        Ok(nomad_client)
    }

    fn get_nomad_token(&self) -> Result<Cow<str>, Error> {
        match self.nomad_token {
            Some(ref token) => Ok(Cow::Borrowed(token.as_str())),
            None => {
                info!("No Nomad Token configured. Retrieving from Vault");
                let vault_client = self.get_vault_client()?;

                let nomad_path = self
                    .vault_config
                    .nomad_path
                    .as_ref()
                    .ok_or_else(|| Error::MissingConfiguration("nomad_path".to_string()))?;
                let nomad_role = self
                    .vault_config
                    .nomad_role
                    .as_ref()
                    .ok_or_else(|| Error::MissingConfiguration("nomad_role".to_string()))?;

                Ok(Cow::Owned(
                    vault_client.get_nomad_token(nomad_path, nomad_role)?.0,
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
                info!("No Vault Token configured. Using AWS Credentials to retrieve from Vault");
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
    env_logger::init();
    lambda!(lambda_wrapper);
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn lambda_wrapper(event: Event, context: Context) -> Result<HandlerResult, HandlerError> {
    match lambda_handler(&event, &context) {
        Ok(result) => Ok(result),
        Err(e) => {
            let mut error_output = vec![format!("{}", e)];
            if let Some(backtrace) = e.backtrace() {
                error_output.push(format!("Backtrace: {}", backtrace));
            }
            let error_output = error_output.join("\n");
            error!("{}", error_output);
            Err(context.new_error(&error_output))
        }
    }
}

fn lambda_handler(event: &Event, _context: &Context) -> Result<HandlerResult, Error> {
    let config = Config::from_environment()?;

    info!("Configuration loaded: {:#?}", config);
    let nomad_client = config.new_nomad_client()?;

    let asg_event: AsgEventDetails = serde_json::from_value(serde_json::to_value(&event.detail)?)?;
    info!("Event Details: {:#?}", asg_event);

    if asg_event.lifecycle_transition != AsgLifecycleTransition::InstanceTerminating {
        Err(Error::UnexpectedLifecycleTransition)?;
    }

    info!("Instance ID {} is being terminated", asg_event.instance_id);

    let node = nomad_client.find_node_by_instance_id(&asg_event.instance_id)?;

    info!("Setting Node ID {} to be ineligible", node.data.id);
    nomad_client.set_node_eligibility(
        &node.data.id,
        nomad_drain::nomad::NodeEligibility::Ineligible,
    )?;

    info!("Draining Nomad Node ID {}", node.data.id);
    // Lambda has a max runtime of 900s. Let's set a deadline for 600s
    nomad_client.set_node_drain(
        &node.data.id,
        true,
        Some(nomad_drain::nomad::DrainSpec {
            deadline: 600,
            ignore_system_jobs: false,
        }),
    )?;

    info!("Node ID {} Drained", node.data.id);

    info!("Marking lifecycle action complete");
    // Complete the lifecycle action
    let asg_client = AutoscalingClient::new(Default::default());
    let _ = asg_client
        .complete_lifecycle_action(CompleteLifecycleActionType {
            auto_scaling_group_name: asg_event.auto_scaling_group_name.to_string(),
            instance_id: Some(asg_event.instance_id.to_string()),
            lifecycle_action_result: "CONTINUE".to_string(),
            lifecycle_action_token: Some(asg_event.lifecycle_action_token.to_string()),
            lifecycle_hook_name: asg_event.lifecycle_hook_name.to_string(),
        })
        .sync()?;

    info!("Lifecycle action complete");

    // Revoke self

    Ok(HandlerResult {
        instance_id: asg_event.instance_id.to_string(),
        node_id: node.data.id.to_string(),
        timestamp: chrono::Utc::now(),
    })
}
