terraform {
  backend "pg" {
    schema_name = "usebits_app"
  }

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "5.11.0"
    }
  }
}

data "terraform_remote_state" "dns" {
  backend = "pg"

  config = {
    schema_name = "dns"
  }
}

provider "cloudflare" {}
