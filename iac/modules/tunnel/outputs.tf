output "tunnel_id" {
  description = "Cloudflare tunnel ID"
  value       = cloudflare_zero_trust_tunnel_cloudflared.main.id
}

output "tunnel_token" {
  description = "Cloudflare tunnel secret for cloudflared agent"
  value       = cloudflare_zero_trust_tunnel_cloudflared.main.tunnel_secret
  sensitive   = true
}

output "tunnel_cname" {
  description = "CNAME target for tunnel"
  value       = "${cloudflare_zero_trust_tunnel_cloudflared.main.id}.cfargotunnel.com"
}
