variable "tunnel_name" {
  description = "Cloudflare tunnel name"
  type        = string
}

variable "cloudflare_account_id" {
  description = "Cloudflare account ID"
  type        = string
}

variable "zone_id" {
  description = "Cloudflare zone ID for DNS records"
  type        = string
}

variable "domain" {
  description = "Domain name (e.g., bits.page)"
  type        = string
}
