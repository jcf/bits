resource "postmark_server" "main" {
  name          = var.name
  color         = var.color
  delivery_type = var.delivery_type
}

resource "postmark_domain" "main" {
  name = var.domain
}

resource "postmark_stream" "transactional" {
  stream_id           = "${var.name}-transactional"
  name                = "Transactional"
  message_stream_type = "Transactional"
  server_token        = postmark_server.main.api_tokens[0]
}
