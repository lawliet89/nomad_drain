# Nomad Node Drain Terraform Module

- Manually allow egress and ingress to the API endpoints
- Optional VPCE for AWS API

## Providers

| Name | Version |
|------|---------|
| aws | n/a |
| nomad | n/a |
| template | n/a |
| vault | n/a |

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:-----:|
| asg\_name | Name of the Nomad Client Autoscaling group | `any` | n/a | yes |
| auth\_path | Path the Vault AWS authentication engine | `string` | `"aws"` | no |
| auth\_role | Name of the Role that the AWS Lambda will use to authenticate with Vault | `string` | `"nomad_drain_lambda"` | no |
| aws\_auth\_header\_value | Header value that must be included when authenticating via AWS, if set | `string` | `""` | no |
| enable\_backtrace | Enable backtrace generation during errors | `bool` | `false` | no |
| lambda\_description | Lanbda description text | `string` | `"Automatically drain a Nomad node of allocations when the instance is terminating."` | no |
| lambda\_name | Name of the Nomad Drain Lambda | `string` | `"nomad_node_drain"` | no |
| lambda\_payload | Path to the Lambda payload. The payload must be zipped with the binary named `bootstrap`. It must be compiled for the target `x86_64-unknown-linux-musl` | `any` | n/a | yes |
| lambda\_timeout | Lambda Timeout in seconds. Maximum is 900 | `number` | `900` | no |
| lifecycle\_hook\_name | Name of the lifecycle hook | `string` | `"nomad_client_drain"` | no |
| lifecycle\_hook\_timeout | Maximum amount of time the lifecycle hook is allowed to run in seconds. | `number` | `900` | no |
| log\_level | Log level for the Lambda. Refer to https://docs.rs/env_logger/0.6.0/env_logger/#enabling-logging for details. | `string` | `"nomad_drain=info,bootstrap=info"` | no |
| nomad\_acl\_policy\_name | Name of the Nomad ACL policy to allow the Lambda to drain nodes | `string` | `"LambdaDrain"` | no |
| nomad\_address | Address to Nomad Server API | `any` | n/a | yes |
| nomad\_path | Path to the Vault's Nomad secrets engine | `string` | `"nomad"` | no |
| nomad\_role | Name of the role for the lambda to retrieve Nomad Token | `string` | `"nomad_drain_lambda"` | no |
| notification\_metadata | Additional Metadata to pass to the Lambda on notification | `string` | `""` | no |
| tags | Map of tags for resources | `map` | <pre>{<br>  "Terraform": "true"<br>}<br></pre> | no |
| vault\_address | Address to Vault API | `any` | n/a | yes |
| vault\_policy\_name | Name of the Vault Policy to allow the lambda to retrieve Nomad tokens | `string` | `"nomad_drain_lambda"` | no |
| vpc\_id | VPC ID to run the lambda in | `any` | n/a | yes |
| vpc\_subnets | VPC Subnet IDs to run the lambda in | `list(string)` | n/a | yes |

## Outputs

| Name | Description |
|------|-------------|
| lambda\_iam\_role | Name of the IAM Role for the lambda |
| lambda\_security\_group\_id | Security Group ID for the Lambda Function |
