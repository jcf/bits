resource "hcloud_server" "main" {
  name        = var.server_name
  server_type = var.server_type
  location    = var.location
  image       = "ubuntu-24.04"

  user_data = templatefile("${path.module}/cloud-init.yml.tftpl", {
    tailscale_authkey = var.tailscale_authkey
    tunnel_token      = var.cloudflare_tunnel_token
    server_name       = var.server_name
    ssh_keys          = yamlencode(var.ssh_keys)
  })

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
