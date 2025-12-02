resource "cloudflare_dns_record" "dkim" {
  zone_id = var.zone_id
  name    = var.dkim_host
  type    = "TXT"
  content = var.dkim_text_value
  ttl     = var.ttl
}

resource "cloudflare_dns_record" "return_path" {
  zone_id = var.zone_id
  name    = var.return_path_domain
  type    = "CNAME"
  content = var.return_path_cname_value
  ttl     = var.ttl
}
