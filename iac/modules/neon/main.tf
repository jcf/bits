resource "neon_project" "main" {
  name = var.project_name

  default_endpoint_settings {
    autoscaling_limit_min_cu = 0.25
    autoscaling_limit_max_cu = 0.25
  }
}

resource "neon_database" "main" {
  project_id = neon_project.main.id
  branch_id  = neon_project.main.default_branch_id
  name       = "bits"
  owner_name = neon_project.main.database_user
}
