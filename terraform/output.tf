output "lambda_security_group_id" {
  description = "Security Group ID for the Lambda Function"
  value       = aws_security_group.lambda.id
}

output "lambda_iam_role" {
  description = "Name of the IAM Role for the lambda"
  value       = aws_iam_role.lambda.name
}
