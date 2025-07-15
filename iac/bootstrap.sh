#!/usr/bin/env bash
set -e

echo "Bootstrapping Terraform backend for invetica-bits..."

# Create DynamoDB table for state locking
echo "Creating DynamoDB table for state locking..."
aws-vault exec bits -- aws dynamodb create-table \
  --table-name terraform-state-lock \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
  --region eu-west-2 \
  2>/dev/null || echo "DynamoDB table already exists"

# Enable versioning on the state bucket
echo "Enabling versioning on state bucket..."
aws-vault exec bits -- aws s3api put-bucket-versioning \
  --bucket invetica-bits-terraform-state \
  --versioning-configuration Status=Enabled \
  --region eu-west-2

echo "Bootstrap complete!"