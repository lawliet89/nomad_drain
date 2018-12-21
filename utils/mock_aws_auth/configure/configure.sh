#!/bin/bash
set -euo pipefail

export AWS_ACCESS_KEY_ID=xxx
export AWS_SECRET_ACCESS_KEY=xxx

export VAULT_TOKEN=0000000000
export VAULT_ADDR=http://127.0.0.1:8200
HEADER_VALUE="vault.example.com"

ROLE_ARN="arn:aws:iam::123456789012:user/*"
IAM_ENDPOINT="http://127.0.0.1:5001"

aws --endpoint-url "${IAM_ENDPOINT}" iam create-user --user-name moto

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
