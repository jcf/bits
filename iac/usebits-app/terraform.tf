terraform {
  backend "pg" {
    conn_str    = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
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
    conn_str    = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
    schema_name = "dns"
  }
}

provider "cloudflare" {}
