variable "zone_id" {
  type        = string
  description = "Cloudflare zone ID"
}

variable "dkim_host" {
  type        = string
  description = "DKIM DNS record host"
  sensitive   = true
}

variable "dkim_text_value" {
  type        = string
  description = "DKIM DNS record value"
  sensitive   = true
}

variable "return_path_domain" {
  type        = string
  description = "Return path domain"
  sensitive   = true
}

variable "return_path_cname_value" {
  type        = string
  description = "Return path CNAME value"
  sensitive   = true
}

# ------------------------------------------------------------------------------
# Optional

variable "ttl" {
  type        = number
  description = "DNS record TTL"
  default     = 3600
}
