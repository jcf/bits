output "project_id" {
  description = "Neon project ID"
  value       = neon_project.main.id
}

output "database_name" {
  description = "Database name"
  value       = neon_database.main.name
}

output "connection_url" {
  description = "Pooled connection URL for application use"
  value       = neon_project.main.connection_uri
  sensitive   = true
}

output "direct_url" {
  description = "Direct connection URL for migrations"
  value       = neon_project.main.connection_uri
  sensitive   = true
}

output "database_user" {
  description = "Database user"
  value       = neon_project.main.database_user
}
