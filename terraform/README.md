# Nomad Node Drain Terraform Module

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|:----:|:-----:|:-----:|
| asg_name | Name of the Nomad Client Autoscaling group | string | - | yes |
| auth_path | Path the Vault AWS authentication engine | string | `aws` | no |
| auth_role | Name of the Role that the AWS Lambda will use to authenticate with Vault | string | `nomad_drain_lambda` | no |
| aws_auth_header_value | Header value that must be included when authenticating via AWS, if set | string | `` | no |
| lambda_description | Lanbda description text | string | `Automatically drain a Nomad node of allocations when the instance is terminating.` | no |
| lambda_name | Name of the Nomad Drain Lambda | string | `nomad_node_drain` | no |
| lambda_timeout | Lambda Timeout in seconds. Maximum is 900 | string | `900` | no |
| lifecycle_hook_name | Name of the lifecycle hook | string | `nomad_client_drain` | no |
| nomad_acl_policy_name | Name of the Nomad ACL policy to allow the Lambda to drain nodes | string | `nomad_drain_lambda` | no |
| nomad_address | Address to Nomad Server API | string | - | yes |
| nomad_api_port | Port for the Nomad API | string | `4646` | no |
| nomad_path | Path to the Vault's Nomad secrets engine | string | `nomad` | no |
| nomad_role | Name of the role for the lambda to retrieve Nomad Token | string | `nomad_drain_lambda` | no |
| nomad_server_security_group | Security Group ID for Nomad servers | string | - | yes |
| tags | Map of tags for resources | string | `<map>` | no |
| vault_address | Address to Vault API | string | - | yes |
| vault_api_port | Port for Vault API | string | `8200` | no |
| vault_policy_name | Name of the Vault Policy to allow the lambda to retrieve Nomad tokens | string | `nomad_drain_lambda` | no |
| vault_security_group | Security Group ID for Vault | string | - | yes |
| vpc_id | VPC ID to run the lambda in | string | - | yes |
| vpc_subnets | VPC Subnet IDs to run the lambda in | list | - | yes |

## Outputs

| Name | Description |
|------|-------------|
| lambda_security_group_id | Security Group ID for the Lambda Function |
