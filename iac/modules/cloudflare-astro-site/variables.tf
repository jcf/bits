variable "account_id" {
  description = "Cloudflare account ID"
  type        = string
}

variable "zone_id" {
  description = "Cloudflare zone ID"
  type        = string
}

variable "domain_name" {
  description = "Primary domain name"
  type        = string
}

variable "project_name" {
  description = "Cloudflare Pages project name"
  type        = string
}

variable "github_owner" {
  description = "GitHub repository owner"
  type        = string
}

variable "github_repo" {
  description = "GitHub repository name"
  type        = string
}

variable "build_command" {
  default = "pnpm build"
}

variable "destination_dir" {
  default = "dist"
}

variable "root_dir" {
  default = ""
}

variable "domains" {
  description = "List of domains to attach to the Pages project"
  type        = set(string)
}

variable "canonical_domain" {
  description = "Canonical domain to redirect to (e.g., www.example.com)"
  type        = string
}

variable "compatibility_date" {
  description = "Cloudflare Workers compatibility date"
  type        = string
  default     = "2024-09-02"
}

variable "extra_env_vars" {
  description = "Additional environment variables for both preview and prod"
  type = map(object({
    type  = string
    value = string
  }))
  default = {}
}

variable "extra_prod_env_vars" {
  description = "Additional environment variables for prod only"
  type = map(object({
    type  = string
    value = string
  }))
  default = {}
}
