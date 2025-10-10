locals {
  # Determine redirect source (non-canonical domain)
  redirect_from = var.canonical_domain == var.domain_name ? "www.${var.domain_name}" : var.domain_name

  # Standard Astro env vars (always present in both preview and prod)
  standard_env_vars = {
    SITE_NAME = {
      type  = "plain_text"
      value = var.domain_name
    }
    ASTRO_SITE = {
      type  = "plain_text"
      value = "https://${var.canonical_domain}"
    }
  }

  # Analytics only in prod
  prod_only_env_vars = {
    PUBLIC_ANALYTICS_DOMAIN = {
      type  = "plain_text"
      value = var.canonical_domain
    }
  }

  # Merge: standard + extra for preview
  preview_env_vars = merge(local.standard_env_vars, var.extra_env_vars)

  # Merge: standard + analytics + extra + extra_prod for prod
  prod_env_vars = merge(
    local.standard_env_vars,
    local.prod_only_env_vars,
    var.extra_env_vars,
    var.extra_prod_env_vars
  )
}

resource "cloudflare_pages_project" "main" {
  account_id        = var.account_id
  name              = var.project_name
  production_branch = "main"

  build_config = {
    build_caching   = true
    build_command   = var.build_command
    destination_dir = var.destination_dir
    root_dir        = var.root_dir
  }

  source = {
    type = "github"
    config = {
      owner                         = var.github_owner
      repo_name                     = var.github_repo
      production_branch             = "main"
      pr_comments_enabled           = true
      deployments_enabled           = true
      production_deployment_enabled = true
      preview_deployment_setting    = "all"
      preview_branch_includes       = ["*"]
      preview_branch_excludes       = []
    }
  }

  deployment_configs = {
    preview = {
      compatibility_date  = var.compatibility_date
      compatibility_flags = ["nodejs_compat"]
      env_vars            = local.preview_env_vars
    }

    production = {
      compatibility_date  = var.compatibility_date
      compatibility_flags = ["nodejs_compat"]
      env_vars            = local.prod_env_vars
    }
  }

  lifecycle {
    ignore_changes = [source]
  }
}

resource "cloudflare_pages_domain" "main" {
  for_each = var.domains

  account_id   = var.account_id
  project_name = cloudflare_pages_project.main.name
  name         = each.value

  lifecycle {
    ignore_changes = [account_id, project_name]
  }
}

resource "cloudflare_dns_record" "apex" {
  zone_id = var.zone_id
  name    = "@"
  type    = "CNAME"
  content = cloudflare_pages_project.main.subdomain
  ttl     = 1
  proxied = true
}

resource "cloudflare_dns_record" "www" {
  zone_id = var.zone_id
  name    = "www"
  type    = "CNAME"
  content = cloudflare_pages_project.main.subdomain
  ttl     = 1
  proxied = true
}

resource "cloudflare_ruleset" "redirect" {
  zone_id     = var.zone_id
  name        = "Redirect ${local.redirect_from} to ${var.canonical_domain}"
  description = "Redirect ${local.redirect_from} to ${var.canonical_domain}"
  kind        = "zone"
  phase       = "http_request_dynamic_redirect"

  rules = [{
    action = "redirect"
    action_parameters = {
      from_value = {
        status_code = 301
        target_url = {
          expression = "concat(\"https://${var.canonical_domain}\", http.request.uri.path)"
        }
        preserve_query_string = true
      }
    }
    expression  = "http.host eq \"${local.redirect_from}\""
    description = "Redirect ${local.redirect_from} to ${var.canonical_domain}"
    enabled     = true
  }]
}
