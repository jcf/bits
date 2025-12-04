resource "cloudflare_r2_bucket" "images" {
  account_id = var.cloudflare_account_id
  name       = "${var.r2_bucket_prefix}-images"
  location   = "enam"
}

resource "cloudflare_r2_bucket" "videos" {
  account_id = var.cloudflare_account_id
  name       = "${var.r2_bucket_prefix}-videos"
  location   = "enam"
}
