data "template_file" "nomad_lambda_role" {
  template = "${file("${path.module}/vault/nomad_role.json")}"

  vars {
    lambda_policy_name = "${nomad_acl_policy.lambda.name}"
  }
}

resource "vault_generic_secret" "nomad_lambda_role" {
  path      = "${var.nomad_path}/role/${var.nomad_role}"
  data_json = "${data.template_file.nomad_lambda_role.rendered}"
}

data "template_file" "vault_policy" {
  template = "${file("${path.module}/vault/policy.hck")}"

  vars {
    nomad_path = "${var.nomad_path}"
    nomad_role = "${var.nomad_role}"
  }
}

resource "vault_policy" "nomad_lambda" {
  name   = "${var.vault_policy_name}"
  policy = "${data.template_file.vault_policy.rendered}"
}

resource "vault_aws_auth_backend_role" "lambda" {
  backend                  = "${var.auth_path}"
  role                     = "${var.auth_role}"
  auth_type                = "iam"
  bound_iam_principal_arns = ["${aws_iam_role.lambda.arn}"]
  ttl                      = "${var.lambda_timeout}"
  max_ttl                  = "${var.lambda_timeout}"
  policies                 = ["${vault_policy.nomad_lambda.name}"]
}
