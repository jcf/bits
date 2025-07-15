terraform {
  required_version = ">= 1.6"
}

locals {
  environment = "dev"
  aws_region  = "eu-west-2"
}

module "storage" {
  source = "../../modules/storage"

  environment = local.environment
  bucket_name = "bits-${local.environment}-content"
}

output "s3_bucket_name" {
  value = module.storage.bucket_name
}

output "aws_access_key_id" {
  value = module.storage.access_key_id
}

output "aws_secret_access_key" {
  value     = module.storage.secret_access_key
  sensitive = true
}