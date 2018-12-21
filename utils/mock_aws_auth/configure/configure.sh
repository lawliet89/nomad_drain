#!/bin/bash
set -euo pipefail

export AWS_ACCESS_KEY_ID=xxx
export AWS_SECRET_ACCESS_KEY=xxx

export VAULT_TOKEN=0000000000
export VAULT_ADDR=http://vault:8200
HEADER_VALUE="vault.example.com"

ROLE_ARN="arn:aws:iam::123456789012:user/*"
IAM_ENDPOINT="aws_iam"
IAM_ENDPOINT=$(getent hosts "${IAM_ENDPOINT}" | awk '{ print $1 }' | head -1)

aws --endpoint-url "http://${IAM_ENDPOINT}:5000" --region us-east-1 iam create-user --user-name moto

vault auth enable aws
vault write auth/aws/config/client \
    secret_key=aaa \
    access_key=aaa \
    iam_endpoint=http://aws_iam:5000 \
    sts_endpoint=http://aws_sts:8000 \
    iam_server_id_header_value="${HEADER_VALUE}"

vault write auth/aws/role/default \
    auth_type=iam \
    bound_iam_principal_arn="${ROLE_ARN}" \
    policies=default

echo "Configuration Done. You can now run tests."
