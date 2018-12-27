# Reference: https://docs.aws.amazon.com/autoscaling/ec2/userguide/lifecycle-hooks.html
locals {
  lambda_payload_path = "${path.module}/payload/lambda.zip"
}

data "aws_autoscaling_group" "asg" {
  name = "${var.asg_name}"
}

data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "lambda" {
  name = "${var.lambda_name}"

  assume_role_policy = "${data.aws_iam_policy_document.lambda_assume_role.json}"
}

resource "aws_security_group" "lambda" {
  name                   = "${var.lambda_name}"
  description            = "Security group for the ${var.lambda_name} lambda function"
  vpc_id                 = "${var.vpc_id}"
  revoke_rules_on_delete = true

  tags = "${merge(var.tags, map("Name", var.lambda_name))}"
}

resource "aws_security_group_rule" "nomad" {
  type                     = "egress"
  from_port                = "${var.nomad_api_port}"
  to_port                  = "${var.nomad_api_port}"
  protocol                 = "tcp"
  source_security_group_id = "${var.nomad_server_security_group}"
  description              = "Access Nomad Server API"

  security_group_id = "${aws_security_group.lambda.id}"
}

resource "aws_security_group_rule" "vault" {
  type                     = "egress"
  from_port                = "${var.vault_api_port}"
  to_port                  = "${var.vault_api_port}"
  protocol                 = "tcp"
  source_security_group_id = "${var.vault_security_group}"
  description              = "Access Vault Server API"

  security_group_id = "${aws_security_group.lambda.id}"
}

resource "aws_lambda_function" "drain" {
  filename         = "${local.lambda_payload_path}"
  source_code_hash = "${base64sha256(file("${local.lambda_payload_path}"))}"

  function_name = "${var.lambda_name}"
  role          = "${aws_iam_role.lambda.arn}"
  description   = "${var.lambda_description}"

  handler = "doesnt_matter"
  runtime = "provided"
  timeout = "${var.lambda_timeout}"

  vpc_config {
    subnet_ids         = "${var.vpc_subnets}"
    security_group_ids = ["${aws_security_group.lambda.id}"]
  }

  variables {
    NOMAD_ADDR        = "${var.nomad_address}"
    USE_NOMAD_TOKEN   = "true"
    VAULT_ADDR        = "${var.vault_address}"
    AUTH_PATH         = "${var.auth_path}"
    AUTH_ROLE         = "${vault_aws_auth_backend_role.lambda.role}"
    AUTH_HEADER_VALUE = "${var.aws_auth_header_value}"
    NOMAD_PATH        = "${var.nomad_path}"
    NOMAD_ROLE        = "${var.nomad_role}"
  }

  tags = "${var.tags}"
}

resource "aws_cloudwatch_event_rule" "drain" {
  name        = "${var.lambda_name}"
  description = "Invoke Lambda named ${var.lambda_name} when the ASG ${var.asg_name} terminates instances to drain Nomad nodes of allocations"

  event_pattern = <<EOF
{
  "source": [ "aws.autoscaling" ],
  "detail-type": [ "EC2 Instance-terminate Lifecycle Action" ],
  "resources": [
    "${data.aws_autoscaling_group.asg.arn}"
  ]
}
EOF

  role_arn = ""
}

resource "aws_lambda_permission" "cloudwatch_events" {
  action              = "lambda:InvokeFunction"
  function_name       = "${aws_lambda_function.drain.function_name}"
  principal           = "events.amazonaws.com"
  source_arn          = "${aws_cloudwatch_event_rule.drain.arn}"
  statement_id_prefix = "${var.lambda_name}"
}

resource "aws_cloudwatch_event_target" "drain_lambda" {
  rule = "${aws_cloudwatch_event_rule.drain.name}"
  arn  = "${aws_lambda_function.drain.arn}"
}

data "aws_iam_policy_document" "asg_lifecycle" {
  statement {
    actions = [
      "autoscaling:CompleteLifecycleAction",
    ]

    resources = [
      "${data.aws_autoscaling_group.asg.arn}",
    ]
  }
}

resource "aws_iam_role_policy" "asg_lifecycle" {
  role   = "${aws_iam_role.lambda.id}"
  policy = "${data.aws_iam_policy_document.asg_lifecycle.json}"
}