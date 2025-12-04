variable "ssh_keys" {
  description = "SSH public keys for server access"
  type        = list(string)
  default = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJCFqBB/awADSj49zwaFnni4O1KHedQTax4b/8RyvMfX Max"
  ]
}

variable "tailscale_authkey" {
  description = "Tailscale authentication key"
  type        = string
  sensitive   = true
}
