resource "cloudflare_dns_record" "dkim" {
  for_each = {
    "fm1._domainkey" = "fm1.%s.dkim.fmhosted.com"
    "fm2._domainkey" = "fm2.%s.dkim.fmhosted.com"
    "fm3._domainkey" = "fm3.%s.dkim.fmhosted.com"
  }

  zone_id = var.zone_id
  name    = each.key
  type    = "CNAME"
  content = format(each.value, var.domain_name)
  ttl     = var.ttl
}

resource "cloudflare_dns_record" "mx" {
  for_each = {
    10 = "in1-smtp.messagingengine.com"
    20 = "in2-smtp.messagingengine.com"
  }

  zone_id  = var.zone_id
  name     = "@"
  type     = "MX"
  priority = each.key
  content  = each.value
  ttl      = var.ttl
}

resource "cloudflare_dns_record" "spf" {
  zone_id = var.zone_id
  name    = "@"
  type    = "TXT"
  content = format("\"%s\"", "v=spf1 include:spf.messagingengine.com -all")
  ttl     = var.ttl
}
