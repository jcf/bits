{
  config,
  pkgs,
  modulesPath,
  ...
}: {
  imports = [
    ./hcloud.nix
    (modulesPath + "/profiles/qemu-guest.nix")
  ];

  # Filesystems
  fileSystems."/" = {
    device = "/dev/sda1";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/12CE-A600";
    fsType = "vfat";
  };

  # Boot - systemd-boot for UEFI (Hetzner Cloud standard)
  boot.loader = {
    systemd-boot.enable = true;
    efi.canTouchEfiVariables = true;
  };

  # Networking - DHCP for IPv4, firewall enabled
  networking = {
    useDHCP = true;
    firewall = {
      enable = true;
      # Allow port 8080 for Cloudflare Tunnel to connect to bits-platform
      allowedTCPPorts = [8080];
    };
  };

  # Essential packages
  environment.systemPackages = with pkgs; [
    curl
    git
    htop
    jq
    vim
  ];

  # SSH - Root login with keys only (injected via hcloud metadata)
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "prohibit-password";
      PasswordAuthentication = false;
    };
  };

  # Cloudflare tunnel (configured via cloud-init/user-data in Terraform)
  # The tunnel connects to bits-platform on localhost:8080

  system.stateVersion = "25.05";
}
