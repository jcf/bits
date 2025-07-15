variable "environment" {
  description = "Environment name"
  type        = string
}

variable "bucket_name" {
  description = "S3 bucket name for content storage"
  type        = string
}

resource "aws_s3_bucket" "content" {
  bucket = var.bucket_name
}

resource "aws_s3_bucket_versioning" "content" {
  bucket = aws_s3_bucket.content.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "content" {
  bucket = aws_s3_bucket.content.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "content" {
  bucket = aws_s3_bucket.content.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_cors_configuration" "content" {
  bucket = aws_s3_bucket.content.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "DELETE", "HEAD"]
    allowed_origins = ["*"] # Restrict this in production
    expose_headers  = ["ETag"]
    max_age_seconds = 3000
  }
}

# IAM user for application access
resource "aws_iam_user" "app" {
  name = "bits-${var.environment}-app"
}

resource "aws_iam_access_key" "app" {
  user = aws_iam_user.app.name
}

# Policy for S3 access
resource "aws_iam_user_policy" "app_s3_access" {
  name = "bits-${var.environment}-s3-access"
  user = aws_iam_user.app.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:PutObject",
          "s3:GetObject",
          "s3:DeleteObject",
          "s3:ListBucket"
        ]
        Resource = [
          aws_s3_bucket.content.arn,
          "${aws_s3_bucket.content.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "ses:SendEmail",
          "ses:SendRawEmail"
        ]
        Resource = "*"
      }
    ]
  })
}

output "bucket_name" {
  value = aws_s3_bucket.content.id
}

output "bucket_arn" {
  value = aws_s3_bucket.content.arn
}

output "access_key_id" {
  value = aws_iam_access_key.app.id
}

output "secret_access_key" {
  value     = aws_iam_access_key.app.secret
  sensitive = true
}