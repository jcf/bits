terraform {
  backend "pg" {
    conn_str    = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
    schema_name = "postmark"
  }

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "5.11.0"
    }
    postmark = {
      source  = "jcf/postmark"
      version = "1.2.2"
    }
  }
}

provider "cloudflare" {}
provider "postmark" {}
