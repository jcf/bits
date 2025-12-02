# ------------------------------------------------------------------------------
# Sender Signature (shared)

output "sender_signature_id" {
  description = "Sender signature ID"
  value       = postmark_sender_signature.main.id
}

output "sender_signature_email" {
  description = "Sender signature email address"
  value       = postmark_sender_signature.main.from_email
}

output "sender_signature_confirmed" {
  description = "Whether the sender signature has been confirmed"
  value       = postmark_sender_signature.main.confirmed
}

# ------------------------------------------------------------------------------
# Development

output "dev_server_id" {
  description = "Development Postmark server ID"
  value       = postmark_server.dev.id
}

output "dev_server_api_token" {
  description = "Development Postmark server API token"
  value       = postmark_server.dev.api_tokens[0]
  sensitive   = true
}

output "dev_transactional_stream_token" {
  description = "Development transactional stream API token"
  value       = postmark_server.dev.api_tokens[0]
  sensitive   = true
}

# ------------------------------------------------------------------------------
# Production

output "prod_server_id" {
  description = "Production Postmark server ID"
  value       = postmark_server.prod.id
}

output "prod_server_api_token" {
  description = "Production Postmark server API token"
  value       = postmark_server.prod.api_tokens[0]
  sensitive   = true
}

output "prod_transactional_stream_token" {
  description = "Production transactional stream API token"
  value       = postmark_server.prod.api_tokens[0]
  sensitive   = true
}
