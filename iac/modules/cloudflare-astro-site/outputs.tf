output "project_id" {
  description = "Cloudflare Pages project ID"
  value       = cloudflare_pages_project.main.id
}

output "project_name" {
  description = "Cloudflare Pages project name"
  value       = cloudflare_pages_project.main.name
}

output "subdomain" {
  description = "Cloudflare Pages subdomain"
  value       = cloudflare_pages_project.main.subdomain
}

output "domains" {
  description = "Domains attached to the project"
  value       = var.domains
}
