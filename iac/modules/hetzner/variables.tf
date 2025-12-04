variable "server_name" {
  description = "Server name"
  type        = string
}

# Hetzner Cloud Server Pricing
# https://www.hetzner.com/cloud
#
# All plans include 20TB traffic (30-60TB for CCX) + 1 IPv4 address
#
# COST-OPTIMIZED (Shared vCPU)
# ┌────────┬───────────┬────────┬────────┬──────────┬──────────┬────────────┐
# │ Type   │ CPU       │ vCPUs  │ RAM    │ Disk     │ $/month  │ $/hour     │
# ├────────┼───────────┼────────┼────────┼──────────┼──────────┼────────────┤
# │ CX23   │ x86       │ 2      │ 4 GB   │ 40 GB    │ $4.91    │ $0.0079    │
# │ CAX11  │ ARM       │ 2      │ 4 GB   │ 40 GB    │ $5.51    │ $0.0089    │
# │ CX33   │ x86       │ 4      │ 8 GB   │ 80 GB    │ $7.91    │ $0.0127    │
# │ CAX21  │ ARM       │ 4      │ 8 GB   │ 80 GB    │ $9.11    │ $0.0146    │
# │ CX43   │ x86       │ 8      │ 16 GB  │ 160 GB   │ $12.71   │ $0.0200    │
# │ CAX31  │ ARM       │ 8      │ 16 GB  │ 160 GB   │ $16.91   │ $0.0271    │
# │ CX53   │ x86       │ 16     │ 32 GB  │ 320 GB   │ $23.51   │ $0.0377    │
# │ CAX41  │ ARM       │ 16     │ 32 GB  │ 320 GB   │ $33.11   │ $0.0532    │
# └────────┴───────────┴────────┴────────┴──────────┴──────────┴────────────┘
#
# SHARED PERFORMANCE (Better shared vCPU)
# ┌────────┬───────────┬────────┬────────┬──────────┬──────────┬────────────┐
# │ CPX22  │ AMD       │ 2      │ 4 GB   │ 80 GB    │ $9.11    │ $0.0146    │
# │ CPX32  │ AMD       │ 4      │ 8 GB   │ 160 GB   │ $15.11   │ $0.0242    │
# │ CPX42  │ AMD       │ 8      │ 16 GB  │ 320 GB   │ $27.11   │ $0.0434    │
# │ CPX52  │ AMD       │ 12     │ 24 GB  │ 480 GB   │ $38.51   │ $0.0618    │
# │ CPX62  │ AMD       │ 16     │ 32 GB  │ 640 GB   │ $52.31   │ $0.0839    │
# └────────┴───────────┴────────┴────────┴──────────┴──────────┴────────────┘
#
# DEDICATED (Dedicated vCPU)
# ┌────────┬───────────┬────────┬────────┬──────────┬──────────┬────────────┐
# │ CCX13  │ AMD       │ 2      │ 8 GB   │ 80 GB    │ $16.91   │ $0.0271    │
# │ CCX23  │ AMD       │ 4      │ 16 GB  │ 160 GB   │ $32.51   │ $0.0522    │
# │ CCX33  │ AMD       │ 8      │ 32 GB  │ 240 GB   │ $64.91   │ $0.1040    │
# │ CCX43  │ AMD       │ 16     │ 64 GB  │ 360 GB   │ $129.11  │ $0.2070    │
# │ CCX53  │ AMD       │ 32     │ 128 GB │ 600 GB   │ $256.91  │ $0.4117    │
# │ CCX63  │ AMD       │ 48     │ 192 GB │ 960 GB   │ $384.71  │ $0.6166    │
# └────────┴───────────┴────────┴────────┴──────────┴──────────┴────────────┘
variable "server_type" {
  description = "Hetzner server type"
  type        = string
  default     = "cx23"
}

variable "location" {
  description = "Hetzner location"
  type        = string
  default     = "nbg1"
}

variable "ssh_keys" {
  description = "List of SSH public keys"
  type        = list(string)
  default     = []
}

variable "tailscale_authkey" {
  description = "Tailscale auth key"
  type        = string
  sensitive   = true
}

variable "cloudflare_tunnel_token" {
  description = "Cloudflare tunnel token"
  type        = string
  sensitive   = true
}

variable "cloudflare_account_id" {
  description = "Cloudflare account ID"
  type        = string
}

variable "r2_bucket_prefix" {
  description = "Prefix for R2 bucket names"
  type        = string
}
