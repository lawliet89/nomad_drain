use std::collections::HashMap;

use futures::future::Future;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::param::{Params, ServiceParams};
use rusoto_core::signature::{SignedRequest, SignedRequestPayload};
use rusoto_core::Region;
use rusoto_core::{DefaultCredentialsProvider, ProvideAwsCredentials};
use serde::{Deserialize, Serialize};

// Reference:
// https://github.com/hashicorp/vault/blob/d12547c7faa9c216d1411827bc16606535cb3e61/builtin/credential/aws/path_login.go#L1640
const IAM_SERVER_ID_HEADER: &str = "X-Vault-AWS-IAM-Server-ID";

/// Returns AWS credentials according to the behaviour documented
/// [here](https://rusoto.github.io/rusoto/rusoto_credential/struct.ChainProvider.html).
pub fn credentials() -> Result<AwsCredentials, crate::Error> {
    let provider = DefaultCredentialsProvider::new()?;
    Ok(provider.credentials().wait()?)
}

/// Payload for use when authenticating with Vault AWS Authentication using the IAM method
///
/// See [Vault's Documentation](https://www.vaultproject.io/docs/auth/aws.html#iam-auth-method)
/// for more information.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct VaultAwsAuthIamPayload {
    /// HTTP method used in the signed request. Currently only `POST` is supported
    pub iam_http_request_method: String,
    /// Base64-encoded HTTP URL used in the signed request
    pub iam_request_url: String,
    /// Base64-encoded body of the signed request
    pub iam_request_body: String,
    /// Headers of the signed request
    pub iam_request_headers: HashMap<String, Vec<String>>,
}

impl VaultAwsAuthIamPayload {
    /// Create a payload for use with Vault AWS Authentication using the IAM method
    ///
    /// If the Vault AWS Authentication method has the
    /// [`iam_server_id_header_value`](https://www.vaultproject.io/api/auth/aws/index.html#iam_server_id_header_value)
    /// configured, you *must* provide the configured value in the `header_value` parameter.
    pub fn new(
        credentials: &AwsCredentials,
        header_value: Option<&str>,
        region: Option<Region>,
    ) -> Self {
        let region = region.unwrap_or_default();

        // Code below is referenced from the code for
        // https://rusoto.github.io/rusoto/rusoto_sts/trait.Sts.html#tymethod.get_caller_identity

        // Additional processing for Vault is referenced from Vault CLI's source code:
        // https://github.com/hashicorp/vault/blob/master/builtin/credential/aws/cli.go

        let mut request = SignedRequest::new("POST", "sts", &region, "/");
        let mut params = Params::new();

        params.put("Action", "GetCallerIdentity");
        params.put("Version", "2011-06-15");
        request.set_payload(Some(
            serde_urlencoded::to_string(&params).unwrap().into_bytes(),
        ));
        request.set_content_type("application/x-www-form-urlencoded".to_owned());

        if let Some(value) = header_value {
            request.add_header(IAM_SERVER_ID_HEADER, value);
        }

        request.sign_with_plus(credentials, true);

        let uri = format!(
            "{}://{}{}",
            request.scheme(),
            request.hostname(),
            request.canonical_path()
        );

        let payload = match request.payload {
            Some(SignedRequestPayload::Buffer(ref buffer)) => base64::encode(buffer),
            _ => unreachable!("Payload was set above"),
        };

        // We need to convert the headers from bytes back into Strings...
        let headers = request
            .headers
            .iter()
            .map(|(k, v)| {
                let values = v
                    .iter()
                    .map(|v| unsafe { String::from_utf8_unchecked(v.to_vec()) })
                    .collect();

                (k.to_string(), values)
            })
            .collect();

        Self {
            iam_http_request_method: "POST".to_string(),
            iam_request_url: base64::encode(&uri),
            iam_request_body: payload,
            iam_request_headers: headers,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    // mock_key, mock_secret
    pub(crate) fn credentials() -> Result<AwsCredentials, crate::Error> {
        let provider = rusoto_mock::MockCredentialsProvider;
        Ok(provider.credentials().wait()?)
    }

    pub(crate) fn region() -> Region {
        Region::UsEast1
    }

    pub(crate) fn vault_aws_iam_payload(
        header_value: Option<&str>,
    ) -> Result<VaultAwsAuthIamPayload, crate::Error> {
        let cred = credentials()?;
        Ok(VaultAwsAuthIamPayload::new(
            &cred,
            header_value,
            Some(region()),
        ))
    }

    #[test]
    fn vault_aws_iam_payload_has_expected_values() -> Result<(), crate::Error> {
        let payload = vault_aws_iam_payload(Some("vault.example.com"))?;

        assert_eq!(payload.iam_http_request_method, "POST");
        assert_eq!(
            payload.iam_request_url,
            base64::encode(&format!("https://sts.{}.amazonaws.com/", region().name()))
        );
        assert_eq!(
            payload.iam_request_body,
            base64::encode("Action=GetCallerIdentity&Version=2011-06-15")
        );
        assert!(payload.iam_request_headers.contains_key("authorization"));
        assert_eq!(
            payload
                .iam_request_headers
                .get(&IAM_SERVER_ID_HEADER.to_lowercase()),
            Some(&vec!["vault.example.com".to_string()])
        );
        Ok(())
    }
}
