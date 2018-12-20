// ENV: https://www.vaultproject.io/docs/commands/#environment-variables
use std::borrow::Cow;

use reqwest::{Certificate, Client, ClientBuilder};
use serde_derive::{Deserialize, Serialize};

use crate::Error;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct VaultAwsIamLoginPayload<'a, 'b> {
    pub role: &'a str,
    #[serde(borrow, flatten)]
    pub aws_payload: Cow<'b, crate::aws::VaultAwsAuthIamPayload>,
}

/// Login with AWS IAM authentication method
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
) -> Result<(), Error> {
    let request = build_login_aws_iam_request(address, path, role, aws_payload, client)?;
    Ok(())
}

fn build_login_aws_iam_request(
    address: &str,
    path: &str,
    role: &str,
    aws_payload: &crate::aws::VaultAwsAuthIamPayload,
    client: Option<Client>,
) -> Result<reqwest::Request, Error> {
    let client = match client {
        Some(client) => client,
        None => ClientBuilder::new().build()?,
    };

    let address = url::Url::parse(address)?;
    let address = address.join(&format!("/v1/auth/{}/login", path))?;
    let payload = VaultAwsIamLoginPayload {
        role,
        aws_payload: Cow::Borrowed(aws_payload),
    };
    Ok(client.post(address).json(&payload).build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;

    fn vault_address() -> String {
        env::var("VAULT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string())
    }

    #[test]
    fn login_aws_iam_request_is_built_properly() -> Result<(), crate::Error> {
        let address = vault_address();
        let aws_payload = crate::aws::tests::vault_aws_iam_payload(None)?;
        let request =
            build_login_aws_iam_request(&address, "aws", "default", &aws_payload, None)?;

        assert_eq!(format!("{}/v1/auth/aws/login", address), request.url().to_string());
        assert_eq!(&reqwest::Method::POST, request.method());

        // Can't test payload

        Ok(())
    }
}
