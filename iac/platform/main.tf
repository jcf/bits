# DNS records for bits.page pointing to Cloudflare tunnel on compute
#
# The tunnel is managed via NixOS in the setup repo. This just creates
# the DNS records to route traffic to it.

locals {
  tunnel_id = "3e293806-aa96-4663-a803-35b22ec208eb"
}

data "cloudflare_zone" "bits_page" {
  filter = {
    name = "bits.page"
  }
}

resource "cloudflare_dns_record" "root" {
  zone_id = data.cloudflare_zone.bits_page.zone_id
  type    = "CNAME"
  name    = "@"
  content = "${local.tunnel_id}.cfargotunnel.com"
  proxied = true
  ttl     = 1 # Auto TTL when proxied
}

resource "cloudflare_dns_record" "wildcard" {
  zone_id = data.cloudflare_zone.bits_page.zone_id
  type    = "CNAME"
  name    = "*"
  content = "${local.tunnel_id}.cfargotunnel.com"
  proxied = true
  ttl     = 1
}
