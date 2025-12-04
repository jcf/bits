terraform {
  backend "pg" {
    conn_str    = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
    schema_name = "platform"
  }

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "5.11.0"
    }
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "1.49.1"
    }
    neon = {
      source  = "kislerdm/neon"
      version = "0.6.3"
    }
    random = {
      source  = "hashicorp/random"
      version = "3.6.3"
    }
  }
}

provider "cloudflare" {}
provider "hcloud" {}
provider "neon" {}
