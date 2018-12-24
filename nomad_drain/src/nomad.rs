use std::borrow::Cow;
use std::collections::HashMap;

use reqwest::{Client, ClientBuilder, RequestBuilder};
use serde::{Deserialize, Serialize};

const NOMAD_AUTH_HEADER: &str = "X-Nomad-Token";

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
#[serde(rename_all = "PascalCase")]
pub struct DrainSpec {
    /// Deadline in seconds
    pub deadline: u64,
    /// Whether system jobs are ignored
    pub ignore_system_jobs: bool,
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

/// Get Information about a specific Node ID
///
/// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
/// CA certificate or require client authentication
pub fn node_details(
    nomad_address: &str,
    node_id: &str,
    nomad_token: Option<&str>,
    client: Option<&Client>,
) -> Result<Node, crate::Error> {
    let client = match client {
        Some(client) => Cow::Borrowed(client),
        None => Cow::Owned(ClientBuilder::new().build()?),
    };

    let request = build_node_details_request(nomad_address, node_id, nomad_token, &client)?;
    let details: Node = client.execute(request)?.json()?;
    Ok(details)
}

/// Build requests to get node details
fn build_node_details_request(
    nomad_address: &str,
    node_id: &str,
    nomad_token: Option<&str>,
    client: &Client,
) -> Result<reqwest::Request, crate::Error> {
    let address = format!("{}/v1/node/{}", nomad_address, node_id);
    let request = client.get(&address);
    let request = add_nomad_token_header(request, nomad_token);
    Ok(request.build()?)
}

/// Return a list of nodes
fn nodes(
    nomad_address: &str,
    nomad_token: Option<&str>,
    client: Option<&Client>,
) -> Result<Vec<NodesInList>, crate::Error> {
    let client = match client {
        Some(client) => Cow::Borrowed(client),
        None => Cow::Owned(ClientBuilder::new().build()?),
    };

    let request = build_nodes_request(nomad_address, nomad_token, &client)?;
    let details = client.execute(request)?.json()?;
    Ok(details)
}

/// Build request to retrieve list of nodes
fn build_nodes_request(
    nomad_address: &str,
    nomad_token: Option<&str>,
    client: &Client,
) -> Result<reqwest::Request, crate::Error> {
    let address = format!("{}/v1/nodes", nomad_address);
    let request = client.get(&address);
    let request = add_nomad_token_header(request, nomad_token);
    Ok(request.build()?)
}

/// Given an AWS Instance ID, find the Node details
///
/// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
/// CA certificate or require client authentication
pub fn find_node_by_instance_id(
    instance_id: &str,
    nomad_address: &str,
    nomad_token: Option<&str>,
    client: Option<&Client>,
) -> Result<Node, crate::Error> {
    let client = match client {
        Some(client) => Cow::Borrowed(client),
        None => Cow::Owned(ClientBuilder::new().build()?),
    };

    let nodes = nodes(nomad_address, nomad_token, Some(&client))?;
    let result = nodes
        .into_iter()
        .filter(|node| node.status == "ready")
        .map(|node| node_details(nomad_address, &node.id, nomad_token, Some(&client)))
        .find(|details| match details {
            Ok(details) => match details.attributes.get("unique.platform.aws.instance-id") {
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

/// Set a node eligibility for receiving new allocations
///
/// You can optionally provide a `reqwest::Client` if you have specific needs like custom root
/// CA certificate or require client authentication
pub fn set_node_eligibility(
    nomad_address: &str,
    node_id: &str,
    eligibility: NodeEligibility,
    nomad_token: Option<&str>,
    client: Option<&Client>,
) -> Result<(), crate::Error> {
    let client = match client {
        Some(client) => Cow::Borrowed(client),
        None => Cow::Owned(ClientBuilder::new().build()?),
    };

    let request = NodeEligibilityRequest {
        node_id,
        eligibility,
    };

    let request =
        build_node_eligibility_request(nomad_address, node_id, &request, nomad_token, &client)?;
    // Request is successful if the response can be deserialized
    let _: NodeEligibilityResponse = client.execute(request)?.json()?;
    Ok(())
}

fn build_node_eligibility_request(
    nomad_address: &str,
    node_id: &str,
    payload: &NodeEligibilityRequest,
    nomad_token: Option<&str>,
    client: &Client,
) -> Result<reqwest::Request, crate::Error> {
    let address = format!("{}/v1/node/{}/eligibility", nomad_address, node_id);
    let request = client.post(&address).json(payload);
    let request = add_nomad_token_header(request, nomad_token);
    Ok(request.build()?)
}

fn add_nomad_token_header(
    request_builder: RequestBuilder,
    nomad_token: Option<&str>,
) -> RequestBuilder {
    match nomad_token {
        Some(token) => request_builder.header(NOMAD_AUTH_HEADER, token),
        None => request_builder,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_fixture() -> &'static str {
        include_str!("../fixtures/nomad_node.json")
    }

    fn nodes_fixture() -> &'static str {
        include_str!("../fixtures/nomad_nodes.json")
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
        let nomad_address = "http://127.0.0.1:4646";
        let client = ClientBuilder::new().build()?;
        let request = build_node_details_request(nomad_address, "id", Some("token"), &client)?;

        assert_eq!(
            format!("{}/v1/node/{}", nomad_address, "id"),
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
}
