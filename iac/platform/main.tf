# Reference DNS configuration from separate environment
data "terraform_remote_state" "dns" {
  backend = "pg"
  config = {
    conn_str    = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
    schema_name = "dns"
  }
}

# Cloudflare account
locals {
  cloudflare_account_id = "bd3b970e3a969227ad515d124aa7e273"
}

# Neon database
module "neon" {
  source       = "../modules/neon"
  project_name = "bits-${terraform.workspace}"
}

# Cloudflare tunnel
module "tunnel" {
  source                = "../modules/tunnel"
  tunnel_name           = "bits-${terraform.workspace}"
  cloudflare_account_id = local.cloudflare_account_id
  zone_id               = data.terraform_remote_state.dns.outputs.cloudflare_zone["bits.page"].id
  domain                = "bits.page"
}

# Hetzner infrastructure + R2 buckets
module "hetzner" {
  source = "../modules/hetzner"

  server_name              = "bits-${terraform.workspace}"
  server_type              = "cx23"
  location                 = "nbg1"
  ssh_keys                 = var.ssh_keys
  tailscale_authkey        = var.tailscale_authkey
  cloudflare_tunnel_token  = module.tunnel.tunnel_token
  cloudflare_account_id    = local.cloudflare_account_id
  r2_bucket_prefix         = "bits-${terraform.workspace}"
}
