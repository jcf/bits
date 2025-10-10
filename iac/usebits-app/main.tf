# Organizations are in closed beta so we're stuck with a single account per
# email address, and without these zones we can't create a Bits email address.
#
# https://developers.cloudflare.com/fundamentals/organizations/
data "cloudflare_account" "main" {
  account_id = "bd3b970e3a969227ad515d124aa7e273"
}

locals {
  cloudflare_zone = data.terraform_remote_state.dns.outputs.cloudflare_zone
}

module "site" {
  source = "../modules/cloudflare-astro-site"

  account_id       = data.cloudflare_account.main.account_id
  zone_id          = local.cloudflare_zone["usebits.app"].id
  domain_name      = "usebits.app"
  project_name     = "usebits"
  canonical_domain = "usebits.app"

  github_owner    = "jcf"
  github_repo     = "bits"
  build_command   = "pnpm build:www"
  destination_dir = "apps/www/dist"

  domains = toset([
    "usebits.app",
    "www.usebits.app",
  ])
}
