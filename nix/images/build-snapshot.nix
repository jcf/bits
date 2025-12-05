# Build NixOS snapshot for Hetzner Cloud using Packer
#
# This builds a complete NixOS system from our flake and creates a Hetzner snapshot.
# The snapshot includes the Bits application and all configuration.
#
# Usage:
#   nix-build nix/images/build-snapshot.nix \
#     --impure \
#     --arg hcloud-token '"$HCLOUD_TOKEN"' \
#     --arg snapshot-name '"nixos-25.05-abc1234"'
{
  pkgs ?
    import <nixpkgs> {
      config.allowUnfree = true;
    },
  hcloud-token,
  snapshot-name,
  snapshot-description ? "NixOS snapshot for Bits platform",
  server-type ? "cx23", # Small server for building
  server-location ? "nbg1", # Nuremberg, Germany
}: let
  # Load the flake from the repository root
  flake = builtins.getFlake (toString ../..);

  # Get the NixOS configuration for production
  nixosConfig = flake.nixosConfigurations.bits-prod;

  # Build the system closure (everything needed to run the system)
  systemClosure = nixosConfig.config.system.build.toplevel;

  # Create a Packer configuration that uses our NixOS closure
  packerConfig = pkgs.writeText "packer.pkr.hcl" ''
    packer {
      required_plugins {
        hcloud = {
          version = ">= 1.0.0"
          source  = "github.com/hashicorp/hcloud"
        }
      }
    }

    variable "hcloud_token" {
      type = string
    }

    source "hcloud" "nixos" {
      token       = var.hcloud_token
      image       = "ubuntu-22.04"
      location    = "${server-location}"
      server_type = "${server-type}"
      ssh_username = "root"
      snapshot_name = "${snapshot-name}"
      snapshot_labels = {
        os      = "nixos"
        version = "25.05"
        app     = "bits"
      }
    }

    build {
      sources = ["source.hcloud.nixos"]

      # Install Nix
      provisioner "shell" {
        inline = [
          "curl -L https://nixos.org/nix/install | sh -s -- --daemon",
          ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh"
        ]
      }

      # Copy the system closure to the build server
      provisioner "file" {
        source      = "${systemClosure}"
        destination = "/tmp/nixos-system"
      }

      # Install the NixOS system
      provisioner "shell" {
        inline = [
          "nix-env -p /nix/var/nix/profiles/system --set /tmp/nixos-system",
          "/nix/var/nix/profiles/system/bin/switch-to-configuration boot",
          "rm -rf /tmp/nixos-system"
        ]
      }
    }
  '';

  # Create a build script that runs Packer
  buildScript = pkgs.writeShellScriptBin "build-snapshot" ''
    set -euo pipefail

    echo "Building NixOS snapshot: ${snapshot-name}"
    echo "System closure: ${systemClosure}"

    # Run Packer
    ${pkgs.packer}/bin/packer init ${packerConfig}
    ${pkgs.packer}/bin/packer build \
      -var "hcloud_token=${hcloud-token}" \
      ${packerConfig}

    echo "✅ Snapshot created: ${snapshot-name}"
  '';
in
  buildScript
