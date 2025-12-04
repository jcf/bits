output "server_id" {
  description = "Hetzner server ID"
  value       = hcloud_server.main.id
}

output "server_ipv4" {
  description = "Server IPv4 address"
  value       = hcloud_server.main.ipv4_address
}

output "server_ipv6" {
  description = "Server IPv6 address"
  value       = hcloud_server.main.ipv6_address
}

output "server_name" {
  description = "Server name"
  value       = hcloud_server.main.name
}

output "r2_images_bucket" {
  description = "R2 images bucket name"
  value       = cloudflare_r2_bucket.images.name
}

output "r2_videos_bucket" {
  description = "R2 videos bucket name"
  value       = cloudflare_r2_bucket.videos.name
}
