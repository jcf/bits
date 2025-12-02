# Organizations are in closed beta so we're stuck with a single account per
# email address, and without these zones we can't create a Bits email address.
#
# https://developers.cloudflare.com/fundamentals/organizations/
data "cloudflare_account" "main" {
  account_id = "bd3b970e3a969227ad515d124aa7e273"
}

resource "cloudflare_zone" "main" {
  for_each = toset(["usebits.app", "bits.page"])
  name     = each.key
  account  = { id = data.cloudflare_account.main.account_id }
}

module "fastmail" {
  for_each    = cloudflare_zone.main
  source      = "../modules/cloudflare-fastmail"
  zone_id     = each.value.id
  domain_name = each.value.name
}

resource "cloudflare_dns_record" "bits_page_spf" {
  zone_id = cloudflare_zone.main["bits.page"].id
  name    = "@"
  type    = "TXT"
  content = format("\"%s\"", "v=spf1 include:spf.messagingengine.com include:spf.mtasv.net -all")
  ttl     = 3600
}

resource "cloudflare_dns_record" "usebits_app_spf" {
  zone_id = cloudflare_zone.main["usebits.app"].id
  name    = "@"
  type    = "TXT"
  content = format("\"%s\"", "v=spf1 include:spf.messagingengine.com -all")
  ttl     = 3600
}
