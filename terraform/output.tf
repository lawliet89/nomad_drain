output "lambda_security_group_id" {
  description = "Security Group ID for the Lambda Function"
  value       = "${aws_security_group.lambda.id}"
}
