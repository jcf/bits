resource "random_id" "tunnel_secret" {
  byte_length = 32
}

resource "cloudflare_zero_trust_tunnel_cloudflared" "main" {
  account_id    = var.cloudflare_account_id
  name          = var.tunnel_name
  tunnel_secret = random_id.tunnel_secret.b64_std
}

resource "cloudflare_dns_record" "root" {
  zone_id = var.zone_id
  name    = "@"
  type    = "CNAME"
  content = "${cloudflare_zero_trust_tunnel_cloudflared.main.id}.cfargotunnel.com"
  ttl     = 1
  proxied = true
}

resource "cloudflare_dns_record" "wildcard" {
  zone_id = var.zone_id
  name    = "*"
  type    = "CNAME"
  content = "${cloudflare_zero_trust_tunnel_cloudflared.main.id}.cfargotunnel.com"
  ttl     = 1
  proxied = true
}
