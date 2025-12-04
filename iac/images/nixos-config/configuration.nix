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

  # Boot - systemd-boot for UEFI (Hetzner Cloud standard)
  boot.loader = {
    systemd-boot.enable = true;
    efi.canTouchEfiVariables = true;
  };

  # Networking - DHCP for IPv4, firewall enabled
  networking = {
    useDHCP = true;
    firewall.enable = true;
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

  system.stateVersion = "25.05";
}
