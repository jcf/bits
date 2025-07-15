terraform {
  backend "s3" {
    bucket         = "invetica-bits-terraform-state"
    key            = "bits/terraform.tfstate"
    region         = "eu-west-2"
    dynamodb_table = "terraform-state-lock"
    encrypt        = true
  }
}