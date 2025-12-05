# Look up the NixOS snapshot by name
data "hcloud_image" "nixos" {
  with_selector = "name=${var.nixos_snapshot_name}"
  most_recent   = true
}

resource "hcloud_server" "main" {
  name        = var.server_name
  server_type = var.server_type
  location    = var.location
  image       = data.hcloud_image.nixos.id

  # NixOS configuration is baked into the snapshot
  # No cloud-init needed - systemd services start automatically

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }
}

resource "hcloud_firewall" "main" {
  name = "${var.server_name}-fw"

  # Allow ICMP for monitoring
  rule {
    direction  = "in"
    protocol   = "icmp"
    source_ips = ["0.0.0.0/0", "::/0"]
  }

  # Deny all other inbound traffic (Tailscale/Cloudflared are outbound)
}

resource "hcloud_firewall_attachment" "main" {
  firewall_id = hcloud_firewall.main.id
  server_ids  = [hcloud_server.main.id]
}
