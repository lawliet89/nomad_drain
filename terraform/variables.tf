variable "asg_name" {
  description = "Name of the Nomad Client Autoscaling group"
}

variable "vpc_id" {
  description = "VPC ID to run the lambda in"
}

variable "vpc_subnets" {
  description = "VPC Subnet IDs to run the lambda in"
  type        = "list"
}

variable "nomad_address" {
  description = "Address to Nomad Server API"
}

variable "vault_address" {
  description = "Address to Vault API"
}

variable "nomad_acl_policy_name" {
  description = "Name of the Nomad ACL policy to allow the Lambda to drain nodes"
  default     = "LambdaDrain"
}

variable "auth_path" {
  description = "Path the Vault AWS authentication engine"
  default     = "aws"
}

variable "auth_role" {
  description = "Name of the Role that the AWS Lambda will use to authenticate with Vault"
  default     = "nomad_drain_lambda"
}

variable "nomad_path" {
  description = "Path to the Vault's Nomad secrets engine"
  default     = "nomad"
}

variable "nomad_role" {
  description = "Name of the role for the lambda to retrieve Nomad Token"
  default     = "nomad_drain_lambda"
}

variable "lifecycle_hook_name" {
  description = "Name of the lifecycle hook"
  default     = "nomad_client_drain"
}

variable "lambda_name" {
  description = "Name of the Nomad Drain Lambda"
  default     = "nomad_node_drain"
}

variable "lambda_description" {
  description = "Lanbda description text"
  default     = "Automatically drain a Nomad node of allocations when the instance is terminating."
}

variable "lambda_payload" {
  description = "Path to the Lambda payload. The payload must be zipped with the binary named `bootstrap`. It must be compiled for the target `x86_64-unknown-linux-musl`"
}


variable "lambda_timeout" {
  description = "Lambda Timeout in seconds. Maximum is 900"
  default     = 900
}

variable "vault_policy_name" {
  description = "Name of the Vault Policy to allow the lambda to retrieve Nomad tokens"
  default     = "nomad_drain_lambda"
}

variable "aws_auth_header_value" {
  description = "Header value that must be included when authenticating via AWS, if set"
  default     = ""
}

variable "log_level" {
  description = "Log level for the Lambda. Refer to https://docs.rs/env_logger/0.6.0/env_logger/#enabling-logging for details."
  default     = "nomad_drain=info,bootstrap=info"
}

variable "lifecycle_hook_timeout" {
  description = "Maximum amount of time the lifecycle hook is allowed to run in seconds."
  default     = 900
}

variable "enable_backtrace" {
  description = "Enable backtrace generation during errors"
  default     = false
}

variable "notification_metadata" {
  description = "Additional Metadata to pass to the Lambda on notification"
  default     = ""
}

variable "tags" {
  description = "Map of tags for resources"

  default {
    Terraform = "true"
  }
}
