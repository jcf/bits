terraform {
  backend "pg" {
    schema_name = "platform"
  }

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "5.11.0"
    }
  }
}

provider "cloudflare" {}
