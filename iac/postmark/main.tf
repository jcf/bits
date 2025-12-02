data "terraform_remote_state" "dns" {
  backend = "pg"

  config = {
    conn_str    = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
    schema_name = "dns"
  }
}

# ------------------------------------------------------------------------------
# Domain (shared across environments)

resource "postmark_domain" "main" {
  name = var.domain
}

# ------------------------------------------------------------------------------
# Sender Signature (shared across environments)

resource "postmark_sender_signature" "main" {
  from_email = "hello@${var.domain}"
  name       = "Bits"

  depends_on = [postmark_domain.main]
}

# ------------------------------------------------------------------------------
# Development Environment

resource "postmark_server" "dev" {
  name          = "bits-dev"
  color         = "turquoise"
  delivery_type = "Sandbox"
}

# ------------------------------------------------------------------------------
# Production Environment

resource "postmark_server" "prod" {
  name          = "bits-prod"
  color         = "green"
  delivery_type = "Live"
}

# ------------------------------------------------------------------------------
# DNS Records (shared across environments)

module "bits-email-policy" {
  source = "../modules/cloudflare-email-policy"

  zone_id                 = data.terraform_remote_state.dns.outputs.cloudflare_zone["bits.page"].id
  dkim_host               = coalesce(postmark_domain.main.dkim_host, postmark_domain.main.dkim_pending_host)
  dkim_text_value         = coalesce(postmark_domain.main.dkim_text_value, postmark_domain.main.dkim_pending_text_value)
  return_path_domain      = postmark_domain.main.return_path_domain
  return_path_cname_value = postmark_domain.main.return_path_domain_cname_value
}
