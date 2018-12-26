use std::collections::HashMap;
use std::time::Duration;

use reqwest::{Client as HttpClient, ClientBuilder, RequestBuilder};
use serde::{Deserialize, Serialize};

const NOMAD_AUTH_HEADER: &str = "X-Nomad-Token";
const NOMAD_INDEX_HEADER: &str = "X-Nomad-Index";

/// Nomad API Client
#[derive(Clone, Debug)]
pub struct Client {
    address: String,
    token: Option<String>,
    client: HttpClient,
}

/// Node details in List of nodes
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct NodesInList {
    pub address: String,
    pub datacenter: String,
    pub drain: bool,
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub status: String,
    pub node_class: String,
    pub scheduling_eligibility: NodeEligibility,
    pub version: String,
    pub drivers: HashMap<String, DriverInfo>,
    pub modify_index: u128,
    pub status_description: String,
}

/// Node Data returned from Nomad API
///
/// [Reference](https://github.com/hashicorp/nomad-java-sdk/blob/master/sdk/src/main/java/com/hashicorp/nomad/apimodel/Node.java)
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Node {
    /// ID of the node
    #[serde(rename = "ID")]
    pub id: String,
    /// Name of the Node
    pub name: String,
    /// Attributes for the node
    pub attributes: HashMap<String, String>,
    /// Computed class of the node
    pub computed_class: String,
    /// Create index
    pub create_index: u128,
    /// Data centre the node is in
    pub datacenter: String,
    /// Whether the node is in a draining state
    pub drain: bool,
    /// Strategy in which the node is draining
    #[serde(default)]
    pub drain_strategy: Option<String>,
    /// Drivers information
    #[serde(default)]
    pub drivers: HashMap<String, DriverInfo>,
    /// HTTP Address
    #[serde(rename = "HTTPAddr")]
    pub http_address: String,
    /// Links information
    #[serde(default)]
    pub links: Option<HashMap<String, String>>,
    /// Metadata
    #[serde(default)]
    pub meta: Option<HashMap<String, String>>,
    /// Modify Index
    pub modify_index: u128,
    /// Reserved resources
    pub reserved: Resource,
    /// Scheduling Eligiblity
    pub scheduling_eligibility: NodeEligibility,
    /// Secret ID
    #[serde(rename = "SecretID")]
    pub secret_id: String,
    /// Status
    pub status: String,
    /// Status Description
    pub status_description: String,
    /// Time status was updated
    pub status_updated_at: u64,
    /// Whether TLS is enabled
    #[serde(rename = "TLSEnabled")]
    tls_enabled: bool,
    // We ignore events
    // /// Events Information
    // #[serde(default)]
    // pub events: Vec<HashMap<String, serde_json::Value>>,
}

/// Node Driver Information
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DriverInfo {
    /// Driver specific attributes
    #[serde(default)]
    pub attributes: Option<HashMap<String, String>>,
    /// Whether the driver is detcted
    pub detected: bool,
    /// Healthy or not
    pub healthy: bool,
    /// Description of health
    pub health_description: String,
    /// Time updated
    pub update_time: chrono::DateTime<chrono::Utc>,
}

/// Node Resource Details
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct Resource {
    /// CPU in MHz
    #[serde(rename = "CPU")]
    pub cpu: u64,
    /// Disk space in MB
    #[serde(rename = "DiskMB")]
    pub disk: u64,
    /// IOPS
    #[serde(rename = "IOPS")]
    pub iops: u64,
    /// Memory in MB
    #[serde(rename = "MemoryMB")]
    pub memory: u64,
    /// Networks
    #[serde(default, rename = "Networks")]
    pub networks: Option<Vec<NetworkResource>>,
}

/// Node Network details
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct NetworkResource {
    /// CIDR of the network
    #[serde(rename = "CIDR")]
    pub cidr: String,
    /// Device name
    #[serde(rename = "Device")]
    pub device: String,
    /// List of dynamic ports
    #[serde(default, rename = "DynamicPorts")]
    pub dynamic_ports: Vec<Port>,
    /// IP Address
    #[serde(rename = "IP")]
    pub ip: String,
    /// Mbits
    #[serde(rename = "MBits")]
    pub mbits: u64,
    /// Reserved Ports
    #[serde(default, rename = "ReservedPorts")]
    pub reserved_ports: Vec<Port>,
}

/// Node Port details
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    /// Label of the port
    pub label: String,
    /// Port number
    pub port: u64,
}

/// Drain Strategy
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DrainStrategy {
    /// Specification for draining
    pub drain_spec: DrainSpec,
    /// Deadline where drain must complete
    pub force_deadline: chrono::DateTime<chrono::Utc>,
}

/// Specification for draining
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(default, rename_all = "PascalCase")]
pub struct DrainSpec {
    /// Deadline in seconds
    pub deadline: u64,
    /// Whether system jobs are ignored
    pub ignore_system_jobs: bool,
}

impl Default for DrainSpec {
    fn default() -> Self {
        Self {
            deadline: 3600, // 1 hour
            ignore_system_jobs: false,
        }
    }
}

/// Node eligibility for scheduling
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Copy)]
#[serde(rename_all = "lowercase")]
pub enum NodeEligibility {
    /// Eligible to receive new allocations
    Eligible,
    /// Ineligible for new allocations
    Ineligible,
}

#[derive(Serialize, Eq, PartialEq, Clone, Debug)]
struct NodeEligibilityRequest<'a> {
    #[serde(rename = "NodeID")]
    pub node_id: &'a str,
    #[serde(rename = "Eligibility")]
    pub eligibility: NodeEligibility,
}

#[derive(Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct NodeEligibilityResponse {
    pub eval_create_index: u128,
    #[serde(rename = "EvalIDs")]
    pub eval_ids: Vec<String>,
    pub index: u128,
    pub node_modify_index: u128,
}

#[derive(Serialize, Eq, PartialEq, Clone, Debug)]
struct NodeDrainRequest<'a, 'b> {
    #[serde(rename = "NodeID")]
    pub node_id: &'a str,
    #[serde(rename = "DrainSpec")]
    pub drain_spec: &'b DrainSpec,
}

// These are the same
type NodeDrainResponse = NodeEligibilityResponse;

/// Nomad Responses that support blocking requests
///
/// See the [documentation](https://www.nomadproject.io/api/index.html#blocking-queries) for more
/// details
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BlockingResponse<T> {
    /// The index indicating the "change ID" for the current response
    pub index: u64,
    /// The actual data of the response
    pub data: T,
}

impl Client {
    /// Create a new Nomad Client
    ///
    /// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
    /// CA certificate or require client authentication.
    /// The default client has a timeout set to 6 minutes to allow supporting Nomad's
    /// [blocking queries](https://www.nomadproject.io/api/index.html#blocking-queries). If you
    /// use your own client, make sure to set this as well.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<S1, S2>(
        address: S1,
        token: Option<S2>,
        client: Option<HttpClient>,
    ) -> Result<Self, crate::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let client = match client {
            Some(client) => client,
            None => ClientBuilder::new()
                .timeout(Some(Duration::from_secs(360)))
                .build()?,
        };

        Ok(Self {
            client,
            address: address.as_ref().to_string(),
            token: token.map(|s| s.as_ref().to_string()),
        })
    }

    /// Returns the Nomad Server Address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Reurns the Nomad Token, if any
    pub fn token(&self) -> Option<&str> {
        self.token.as_ref().map(|s| s.as_str())
    }

    /// Returns the HTTP Client used
    pub fn http_client(&self) -> &HttpClient {
        &self.client
    }

    /// Get Information about a specific Node ID
    ///
    /// Supply the optional parameters to take advantage of
    /// [blocking queries](https://www.nomadproject.io/api/index.html#blocking-queries)
    pub fn node_details(
        &self,
        node_id: &str,
        wait_index: Option<u64>,
        wait_timeout: Option<Duration>,
    ) -> Result<BlockingResponse<Node>, crate::Error> {
        let request = self.build_node_details_request(node_id, wait_index, wait_timeout)?;
        let mut response = self.client.execute(request)?;
        let details = response.json()?;
        Self::make_indexed_response(response, details)
    }

    /// Build requests to get node details
    fn build_node_details_request(
        &self,
        node_id: &str,
        wait_index: Option<u64>,
        wait_timeout: Option<Duration>,
    ) -> Result<reqwest::Request, crate::Error> {
        let address = format!("{}/v1/node/{}", &self.address, node_id);
        let request = self.client.get(&address);
        let request = self.add_nomad_token_header(request);
        let request = Self::add_blocking_requests(request, wait_index, wait_timeout);
        Ok(request.build()?)
    }

    /// Return a list of nodes
    ///
    /// Supply the optional parameters to take advantage of
    /// [blocking queries](https://www.nomadproject.io/api/index.html#blocking-queries)
    fn nodes(
        &self,
        wait_index: Option<u64>,
        wait_timeout: Option<Duration>,
    ) -> Result<BlockingResponse<Vec<NodesInList>>, crate::Error> {
        let request = self.build_nodes_request(wait_index, wait_timeout)?;
        let mut response = self.client.execute(request)?;

        let nodes = response.json()?;

        Self::make_indexed_response(response, nodes)
    }

    /// Build request to retrieve list of nodes
    fn build_nodes_request(
        &self,
        wait_index: Option<u64>,
        wait_timeout: Option<Duration>,
    ) -> Result<reqwest::Request, crate::Error> {
        let address = format!("{}/v1/nodes", &self.address);
        let request = self.client.get(&address);
        let request = self.add_nomad_token_header(request);
        let request = Self::add_blocking_requests(request, wait_index, wait_timeout);
        Ok(request.build()?)
    }

    /// Given an AWS Instance ID, find the Node details
    ///
    /// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
    /// CA certificate or require client authentication
    pub fn find_node_by_instance_id(
        &self,
        instance_id: &str,
    ) -> Result<BlockingResponse<Node>, crate::Error> {
        let nodes = self.nodes(None, None)?;
        let result = nodes
            .data
            .into_iter()
            .filter(|node| node.status == "ready")
            .map(|node| self.node_details(&node.id, None, None))
            .find(|details| match details {
                Ok(details) => match details
                    .data
                    .attributes
                    .get("unique.platform.aws.instance-id")
                {
                    Some(id) => id == instance_id,
                    None => false,
                },
                Err(_) => false,
            });

        let result = result.ok_or_else(|| crate::Error::NomadNodeNotFound {
            instance_id: instance_id.to_string(),
        })?;
        Ok(result?)
    }

    /// Set a node eligibility for receiving new allocations
    ///
    /// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
    /// CA certificate or require client authentication
    pub fn set_node_eligibility(
        &self,
        node_id: &str,
        eligibility: NodeEligibility,
    ) -> Result<(), crate::Error> {
        let request = NodeEligibilityRequest {
            node_id,
            eligibility,
        };

        let request = self.build_node_eligibility_request(node_id, &request)?;
        // Request is successful if the response can be deserialized
        let _: NodeEligibilityResponse = self.client.execute(request)?.json()?;
        Ok(())
    }

    fn build_node_eligibility_request(
        &self,
        node_id: &str,
        payload: &NodeEligibilityRequest,
    ) -> Result<reqwest::Request, crate::Error> {
        let address = format!("{}/v1/node/{}/eligibility", self.address, node_id);
        let request = self.client.post(&address).json(payload);
        let request = self.add_nomad_token_header(request);
        Ok(request.build()?)
    }

    /// Mark the node for draining
    ///
    /// You can optionally specify a `DrainSpec`. If you don't provide one, we will use the default.
    ///
    /// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
    /// CA certificate or require client authentication
    pub fn set_node_drain(
        &self,
        node_id: &str,
        drain_spec: Option<DrainSpec>,
    ) -> Result<(), crate::Error> {
        let drain_spec = drain_spec.unwrap_or_default();
        let payload = NodeDrainRequest {
            node_id,
            drain_spec: &drain_spec,
        };
        let request = self.build_drain_request(node_id, &payload)?;
        // Request is successful if the response can be deserialized
        let _: NodeDrainResponse = self.client.execute(request)?.json()?;
        Ok(())
    }

    fn build_drain_request(
        &self,
        node_id: &str,
        payload: &NodeDrainRequest,
    ) -> Result<reqwest::Request, crate::Error> {
        let address = format!("{}/v1/node/{}/drain", &self.address, node_id);
        let request = self.client.post(&address).json(payload);
        let request = self.add_nomad_token_header(request);
        Ok(request.build()?)
    }

    fn add_nomad_token_header(&self, request_builder: RequestBuilder) -> RequestBuilder {
        match &self.token {
            Some(token) => request_builder.header(NOMAD_AUTH_HEADER, token.as_str()),
            None => request_builder,
        }
    }

    fn add_blocking_requests(
        request_builder: RequestBuilder,
        wait_index: Option<u64>,
        wait_timeout: Option<Duration>,
    ) -> RequestBuilder {
        match wait_index {
            Some(index) => {
                let request_builder = request_builder.query(&[("index", index.to_string())]);
                match wait_timeout {
                    None => request_builder,
                    Some(timeout) => {
                        request_builder.query(&[("wait", timeout.as_secs().to_string())])
                    }
                }
            }
            None => request_builder,
        }
    }

    fn make_indexed_response<T>(
        response: reqwest::Response,
        data: T,
    ) -> Result<BlockingResponse<T>, crate::Error> {
        let index = match response.headers().get(NOMAD_INDEX_HEADER) {
            None => 0,
            Some(index) => index.to_str()?.parse()?,
        };

        Ok(BlockingResponse { data, index })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOMAD_ADDRESS: &str = "http://127.0.0.1:4646";

    fn node_fixture() -> &'static str {
        include_str!("../fixtures/nomad_node.json")
    }

    fn nodes_fixture() -> &'static str {
        include_str!("../fixtures/nomad_nodes.json")
    }

    fn nomad_client() -> Client {
        Client::new(NOMAD_ADDRESS, Some("token"), None).expect("Not to fail")
    }

    #[test]
    fn node_is_deserialized_properly() {
        let node: Node = serde_json::from_str(node_fixture()).unwrap();

        assert_eq!("02802087-8786-fdf6-4497-98445c891fb7", node.id);
    }

    #[test]
    fn nodes_list_is_deserialized_properly() {
        let _: Vec<NodesInList> = serde_json::from_str(nodes_fixture()).unwrap();
    }

    #[test]
    fn build_node_details_request_is_built_properly() -> Result<(), crate::Error> {
        let client = nomad_client();
        let request = client.build_node_details_request(
            "id",
            Some(1234),
            Some(Duration::from_secs(300)),
        )?;

        assert_eq!(
            format!("{}/v1/node/{}?index={}&wait={}", NOMAD_ADDRESS, "id", "1234", "300"),
            request.url().to_string()
        );
        assert_eq!(&reqwest::Method::GET, request.method());

        let actual_token = request.headers().get(NOMAD_AUTH_HEADER);
        assert!(actual_token.is_some());
        assert_eq!("token", actual_token.unwrap());

        Ok(())
    }

    #[test]
    fn node_eligibility_response_is_deserialized_properly() {
        let _: NodeEligibilityResponse =
            serde_json::from_str(include_str!("../fixtures/node_eligibility.json")).unwrap();
    }

    #[test]
    fn node_drain_response_is_deserialized_properly() {
        let _: NodeDrainResponse =
            serde_json::from_str(include_str!("../fixtures/node_drain.json")).unwrap();
    }
}
