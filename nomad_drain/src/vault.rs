// ENV: https://www.vaultproject.io/docs/commands/#environment-variables
use std::borrow::Cow;
use std::collections::HashMap;

use reqwest::{Client, ClientBuilder};
use serde_derive::{Deserialize, Serialize};

/// Generic Vault Response
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Response {
    /// An error response
    Error {
        /// List of errors returned from Vault
        errors: Vec<String>,
    },
    /// A successful response
    Response {
        /// Request UUID
        request_id: String,
        /// Lease ID for secrets
        lease_id: String,
        /// Renewable for secrets
        renewable: bool,
        /// Lease duration for secrets
        lease_duration: u64,
        /// Warnings, if any
        #[serde(default)]
        warnings: Option<Vec<String>>,

        /// Auth data for authentication requests
        #[serde(default)]
        auth: Option<Authentication>,

        /// Data for secrets requests
        #[serde(default)]
        data: Option<HashMap<String, String>>,
        // Missing and ignored fields:
        // - wrap_info
    },
}

/// Authentication data from Vault
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Authentication {
    /// The actual token
    pub client_token: String,
    /// The accessor for the Token
    pub accessor: String,
    /// List of policies for token, including from Identity
    pub policies: Vec<String>,
    /// List of tokens directly assigned to token
    pub token_policies: Vec<String>,
    /// Arbitrary metadata
    pub metadata: HashMap<String, String>,
    /// Lease Duration for the token
    pub lease_duration: u64,
    /// Whether the token is renewable
    pub renewable: bool,
    /// UUID for the entity
    pub entity_id: String,
    /// Type of token
    pub token_type: TokenType,
}

/// Type of token from Vault
/// See [Vault Documentation](https://www.vaultproject.io/docs/concepts/tokens.html#token-types-in-detail)
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Long lived service tokens
    Service,
    /// Short lived batch tokens
    Batch,
}

/// Payload to send to Vault for logging in via AWS IAM
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct AwsIamLoginPayload<'a, 'b> {
    pub role: &'a str,
    #[serde(borrow, flatten)]
    pub aws_payload: Cow<'b, crate::aws::VaultAwsAuthIamPayload>,
}

/// Login with AWS IAM authentication method. Returns a Vault token on success
///
/// - `address`: Address of Vault Server. Include the scheme (e.g. `https`) and the host with an
///    optional port
/// - `path`: Path to the AWS authentication engine. Usually just `aws`.
/// - `role`: Name fo the AWS authentication role
/// - `payload`: Authentication payload from calling `aws::VaultAwsAuthIamPayload::new`
///
/// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
/// CA certificate or require client authentication
pub fn login_aws_iam(
    address: &str,
    path: &str,
    role: &str,
    aws_payload: &crate::aws::VaultAwsAuthIamPayload,
    client: Option<Client>,
) -> Result<String, crate::Error> {
    let client = match client {
        Some(client) => client,
        None => ClientBuilder::new().build()?,
    };

    let request = build_login_aws_iam_request(address, path, role, aws_payload, &client)?;
    let response: Response = client.execute(request)?.json()?;
    match response {
        Response::Error { errors } => Err(crate::Error::InvalidVaultResponse(errors.join("; ")))?,
        Response::Response {
            auth: Some(auth), ..
        } => Ok(auth.client_token),
        _ => Err(crate::Error::InvalidVaultResponse(
            "Missing authentication data".to_string(),
        ))?,
    }
}

fn build_login_aws_iam_request(
    address: &str,
    path: &str,
    role: &str,
    aws_payload: &crate::aws::VaultAwsAuthIamPayload,
    client: &Client,
) -> Result<reqwest::Request, crate::Error> {
    let address = url::Url::parse(address)?;
    let address = address.join(&format!("/v1/auth/{}/login", path))?;
    let payload = AwsIamLoginPayload {
        role,
        aws_payload: Cow::Borrowed(aws_payload),
    };
    Ok(client.post(address).json(&payload).build()?)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use std::env;

    pub(crate) fn vault_address() -> String {
        env::var("VAULT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string())
    }

    #[test]
    fn login_aws_iam_request_is_built_properly() -> Result<(), crate::Error> {
        let address = vault_address();
        let aws_payload = crate::aws::tests::vault_aws_iam_payload(None)?;
        let request = build_login_aws_iam_request(
            &address,
            "aws",
            "default",
            &aws_payload,
            &ClientBuilder::new().build()?,
        )?;

        assert_eq!(
            format!("{}/v1/auth/aws/login", address),
            request.url().to_string()
        );
        assert_eq!(&reqwest::Method::POST, request.method());

        // Can't test payload

        Ok(())
    }

    /// Requires Mock AWS API and Vault server
    /// This test does not verify if the signature from rusoto is correct.
    #[test]
    fn login_aws_with_vault_is_successful() -> Result<(), crate::Error> {
        let address = vault_address();
        let aws_payload = crate::aws::tests::vault_aws_iam_payload(Some("vault.example.com"))?;

        let token = login_aws_iam(&address, "aws", "default", &aws_payload, None)?;
        assert!(token.len() > 0);
        Ok(())
    }
}
