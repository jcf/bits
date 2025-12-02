# ------------------------------------------------------------------------------
# Server

output "server_id" {
  description = "Postmark server ID"
  value       = postmark_server.main.id
}

output "server_api_token" {
  description = "Postmark server API token"
  # For some unknown reason the third-party Postmark provider we're using has
  # `api_tokens` rather than `api_tokens` -- maybe that's what Postmark uses in
  # their API?
  #
  # Either way, this isn't a mistake to be fixed. We're just adhering to
  # upstream expectations.
  value     = postmark_server.main.api_tokens[0]
  sensitive = true
}

output "server_name" {
  description = "Postmark server name"
  value       = postmark_server.main.name
}

# ------------------------------------------------------------------------------
# Domain

output "domain_id" {
  description = "Postmark domain ID"
  value       = postmark_domain.main.id
}

output "domain_name" {
  description = "Postmark domain name"
  value       = postmark_domain.main.name
}

output "dkim_host" {
  description = "DKIM DNS record host"
  value       = postmark_domain.main.dkim_pending_host
  sensitive   = true
}

output "dkim_text_value" {
  description = "DKIM DNS record value"
  value       = postmark_domain.main.dkim_pending_text_value
  sensitive   = true
}

output "return_path_domain" {
  description = "Return path domain"
  value       = postmark_domain.main.return_path_domain
  sensitive   = true
}

output "return_path_cname_value" {
  description = "Return path CNAME value"
  value       = postmark_domain.main.return_path_domain_cname_value
  sensitive   = true
}

# ------------------------------------------------------------------------------
# Stream

output "transactional_stream_id" {
  description = "Transactional stream ID"
  value       = postmark_stream.transactional.stream_id
}

output "transactional_stream_token" {
  description = "Transactional stream API token"
  value       = postmark_stream.transactional.server_token
  sensitive   = true
}
