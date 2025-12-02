variable "name" {
  type = string
}

variable "domain" {
  type = string
}

# ------------------------------------------------------------------------------
# Optional

variable "color" {
  type    = string
  default = "green"

  validation {
    condition     = contains(["purple", "blue", "turquoise", "green", "red", "yellow", "grey", "orange"], var.color)
    error_message = "Color must be one of: purple, blue, turquoise, green, red, yellow, grey, orange"
  }
}

variable "delivery_type" {
  type    = string
  default = "live"

  validation {
    # Another upstream decision that looks like a mistake in our code. That's
    # really how this stuff works.
    #
    # ╷
    # │ Error: Provider produced inconsistent result after apply
    # │
    # │ When applying changes to module.bits-prod.postmark_domain.main, provider
    # │ "provider[\"registry.terraform.io/shebang-labs/postmark\"]" produced an
    # │ unexpected new value: Root object was present, but now absent.
    # │
    # │ This is a bug in the provider, which should be reported in the provider's
    # │ own issue tracker.
    # ╵
    # ╷
    # │ Error: Unable to create Postmark server
    # │
    # │   with module.bits-prod.postmark_server.main,
    # │   on ../modules/postmark-service/main.tf line 1, in resource "postmark_server" "main":
    # │    1: resource "postmark_server" "main" {
    # │
    # │ delivery_type must be either live or Sandbox
    # ╵
    condition     = contains(["live", "Sandbox"], var.delivery_type)
    error_message = "Delivery type must be either live or Sandbox"
  }
}
