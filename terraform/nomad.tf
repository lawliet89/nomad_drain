resource "nomad_acl_policy" "lambda" {
  name        = var.nomad_acl_policy_name
  description = "Allow an AWS Lambda to drain Nomad nodes on termination"
  rules_hcl   = file("${path.module}/nomad/policy.hcl")
}
