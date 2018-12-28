#![deny(clippy::all)]

mod error;

pub mod aws;
pub mod nomad;
pub mod vault;

pub use crate::error::Error;

use std::fmt;
use std::ops::Deref;

use futures::future::Future;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::{DefaultCredentialsProvider, ProvideAwsCredentials, Region};
use serde::{Deserialize, Serialize};

/// A wrapper around a String with custom implementation of Display and Debug to not leak
/// secrets during logging.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Secret(pub String);

impl Deref for Secret {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "***")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "***")
    }
}

impl AsRef<str> for Secret {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Secret(s)
    }
}

/// Use AWS credentials to obtain a token from Vault
///
/// If the Vault AWS Authentication method has the
/// [`iam_server_id_header_value`](https://www.vaultproject.io/api/auth/aws/index.html#iam_server_id_header_value)
/// configured, you *must* provide the configured value in the `header_value` parameter.
///
/// If `region` is `None`, we will infer the Region using the behaviour documented
/// [here](https://rusoto.github.io/rusoto/rusoto_core/region/enum.Region.html#default).
pub fn login_to_vault(
    vault_address: &str,
    vault_auth_path: &str,
    vault_auth_role: &str,
    aws_credentials: &AwsCredentials,
    header_value: Option<&str>,
    region: Option<Region>,
) -> Result<vault::Client, Error> {
    let aws_payload = aws::VaultAwsAuthIamPayload::new(aws_credentials, header_value, region);

    vault::Client::login_aws_iam(
        &vault_address,
        vault_auth_path,
        vault_auth_role,
        &aws_payload,
        None,
    )
}

/// Use the priority documented
/// [here](https://rusoto.github.io/rusoto/rusoto_credential/struct.ChainProvider.html)
/// obtain AWS credentials
pub fn get_aws_credentials() -> Result<AwsCredentials, Error> {
    let provider = DefaultCredentialsProvider::new()?;
    Ok(provider.credentials().wait()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    #[test]
    fn expcted_aws_credentials() -> Result<(), crate::Error> {
        let access_key = "test_key";
        let secret_key = "test_secret";

        env::set_var("AWS_ACCESS_KEY_ID", access_key);
        env::set_var("AWS_SECRET_ACCESS_KEY", secret_key);

        let credentials = get_aws_credentials()?;

        assert_eq!(credentials.aws_access_key_id(), access_key);
        assert_eq!(credentials.aws_secret_access_key(), secret_key);

        Ok(())
    }

    /// Requires Mock server for this test
    #[test]
    fn login_to_vault_is_successful() -> Result<(), crate::Error> {
        let credentials = rusoto_core::credential::StaticProvider::new_minimal(
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        let credentials = credentials.credentials().wait()?;

        let client = login_to_vault(
            &crate::vault::tests::vault_address(),
            "aws",
            "default",
            &credentials,
            Some("vault.example.com"),
            None,
        )?;
        assert!(!client.token().is_empty());

        Ok(())
    }
}
