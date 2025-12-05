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

variable "nixos_snapshot_name" {
  description = "NixOS snapshot name to deploy (e.g., nixos-25.05-b00d3cd)"
  type        = string
}
