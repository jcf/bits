output "neon_connection_url" {
  description = "Pooled database connection URL"
  value       = module.neon.connection_url
  sensitive   = true
}

output "neon_direct_url" {
  description = "Direct database connection URL for migrations"
  value       = module.neon.direct_url
  sensitive   = true
}

output "server_ipv4" {
  description = "Server IPv4 address"
  value       = module.hetzner.server_ipv4
}

output "server_name" {
  description = "Server name"
  value       = module.hetzner.server_name
}

output "tunnel_id" {
  description = "Cloudflare tunnel ID"
  value       = module.tunnel.tunnel_id
}

output "tunnel_token" {
  description = "Cloudflare tunnel token"
  value       = module.tunnel.tunnel_token
  sensitive   = true
}

output "r2_images_bucket" {
  description = "R2 images bucket name"
  value       = module.hetzner.r2_images_bucket
}

output "r2_videos_bucket" {
  description = "R2 videos bucket name"
  value       = module.hetzner.r2_videos_bucket
}
