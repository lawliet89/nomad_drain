// ENV: https://www.vaultproject.io/docs/commands/#environment-variables
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;

use log::{debug, info};
use reqwest::{Client as HttpClient, ClientBuilder};
use serde::{Deserialize, Serialize};

/// Vault API Client
#[derive(Clone, Debug)]
pub struct Client {
    token: crate::Secret,
    address: String,
    client: HttpClient,
}

/// Generic Vault Response
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Response {
    /// An error response
    Error {
        /// List of errors returned from Vault
        errors: Vec<String>,
    },
    /// A successful response
    Response(ResponseData),
}

/// Vault General Response Data
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ResponseData {
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
}

/// Authentication data from Vault
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Authentication {
    /// The actual token
    pub client_token: crate::Secret,
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

impl Client {
    /// Create a new API client from an existing Token
    ///
    /// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
    /// CA certificate or require client authentication
    #[allow(clippy::new_ret_no_self)]
    pub fn new<S1, S2>(
        vault_address: S1,
        vault_token: S2,
        client: Option<HttpClient>,
    ) -> Result<Self, crate::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let client = match client {
            Some(client) => client,
            None => ClientBuilder::new().build()?,
        };

        Ok(Self {
            address: vault_address.as_ref().to_string(),
            token: crate::Secret(vault_token.as_ref().to_string()),
            client,
        })
    }

    /// Returns the Vault Token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the Vault address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns the HTTP Client
    pub fn http_client(&self) -> &HttpClient {
        &self.client
    }

    fn execute_request<T>(client: &HttpClient, request: reqwest::Request) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned + Debug,
    {
        debug!("Executing request: {:#?}", request);
        let mut response = client.execute(request)?;
        debug!("Response received: {:#?}", response);
        let body = response.text()?;
        debug!("Response body: {}", body);
        let result = serde_json::from_str(&body)?;
        debug!("Deserialized body: {:#?}", result);
        Ok(result)
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
        vault_address: &str,
        aws_auth_path: &str,
        aws_auth_role: &str,
        aws_payload: &crate::aws::VaultAwsAuthIamPayload,
        client: Option<HttpClient>,
    ) -> Result<Self, crate::Error> {
        info!(
            "Logging in to Vault with AWS Credentials at path `{}` and role `{}",
            aws_auth_path, aws_auth_role
        );
        let client = match client {
            Some(client) => client,
            None => ClientBuilder::new().build()?,
        };

        let request = Self::build_login_aws_iam_request(
            vault_address,
            aws_auth_path,
            aws_auth_role,
            aws_payload,
            &client,
        )?;
        let response: Response = Self::execute_request(&client, request)?;
        let token = match response {
            Response::Error { errors } => {
                Err(crate::Error::InvalidVaultResponse(errors.join("; ")))?
            }
            Response::Response(ResponseData {
                auth: Some(auth), ..
            }) => Ok(auth.client_token),
            _ => Err(crate::Error::InvalidVaultResponse(
                "Missing authentication data".to_string(),
            )),
        }?;

        info!("Vault authentication successful. Received Vault Token");
        Ok(Self {
            address: vault_address.to_string(),
            token,
            client,
        })
    }

    fn build_login_aws_iam_request(
        vault_address: &str,
        aws_auth_path: &str,
        aws_auth_role: &str,
        aws_payload: &crate::aws::VaultAwsAuthIamPayload,
        client: &HttpClient,
    ) -> Result<reqwest::Request, crate::Error> {
        let vault_address = url::Url::parse(vault_address)?;
        let vault_address = vault_address.join(&format!("/v1/auth/{}/login", aws_auth_path))?;
        let payload = AwsIamLoginPayload {
            role: aws_auth_role,
            aws_payload: Cow::Borrowed(aws_payload),
        };
        Ok(client.post(vault_address).json(&payload).build()?)
    }

    /// Get a token from Nomad Secrets Engine
    ///
    /// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
    /// CA certificate or require client authentication
    pub fn get_nomad_token(
        &self,
        nomad_path: &str,
        nomad_role: &str,
    ) -> Result<crate::Secret, crate::Error> {
        info!(
            "Retrieving Nomad Token from Secrets engine mounted at `{}` with role `{}`",
            nomad_path, nomad_role
        );
        let request = self.build_nomad_token_request(nomad_path, nomad_role)?;
        let response: Response = Self::execute_request(&self.client, request)?;
        Ok(From::from(match response {
            Response::Error { errors } => {
                Err(crate::Error::InvalidVaultResponse(errors.join("; ")))?
            }
            Response::Response(ResponseData {
                data: Some(mut data),
                ..
            }) => data.remove("secret_id").ok_or_else(|| {
                crate::Error::InvalidVaultResponse("Missing Nomad token from response".to_string())
            })?,
            _ => Err(crate::Error::InvalidVaultResponse(
                "Missing secrets data".to_string(),
            ))?,
        }))
    }

    fn build_nomad_token_request(
        &self,
        nomad_path: &str,
        nomad_role: &str,
    ) -> Result<reqwest::Request, crate::Error> {
        let vault_address = url::Url::parse(self.address())?;
        let vault_address =
            vault_address.join(&format!("/v1/{}/creds/{}", nomad_path, nomad_role))?;

        Ok(self
            .client
            .get(vault_address)
            .header("X-Vault-Token", self.token.as_str())
            .build()?)
    }
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
        let aws_payload = crate::aws::tests::vault_aws_iam_payload(None, None)?;
        let request = Client::build_login_aws_iam_request(
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
        let aws_payload =
            crate::aws::tests::vault_aws_iam_payload(Some("vault.example.com"), None)?;

        let client = Client::login_aws_iam(&address, "aws", "default", &aws_payload, None)?;
        assert!(!client.token().is_empty());
        Ok(())
    }

    #[test]
    fn nomad_token_secrets_engine_payload_can_be_deserialized() {
        // Example payload from Nomad Secrets Engine
        // e.g. `vault read nomad/creds/default`
        let json = r#"
{
  "request_id": "xxx4",
  "lease_id": "nomad/creds/default/xxx",
  "lease_duration": 2764800,
  "renewable": true,
  "data": {
    "accessor_id": "accessor",
    "secret_id": "secret"
  },
  "warnings": null
}
"#;
        let data = match serde_json::from_str::<Response>(json).unwrap() {
            Response::Response(ResponseData { data, .. }) => data,
            _ => panic!("Invalid deserialization"),
        };
        let nomad = data.unwrap();
        assert_eq!(nomad["secret_id"], "secret");
    }

    #[test]
    fn nomad_token_request_is_built_properly() -> Result<(), crate::Error> {
        let client = Client::new(vault_address(), "vault_token", None)?;
        let request = client.build_nomad_token_request("nomad", "default")?;

        assert_eq!(
            format!("{}/v1/nomad/creds/default", vault_address()),
            request.url().to_string()
        );
        assert_eq!(&reqwest::Method::GET, request.method());

        let actual_token = request.headers().get("X-Vault-Token");
        assert!(actual_token.is_some());
        assert_eq!("vault_token", actual_token.unwrap());

        Ok(())
    }
}
