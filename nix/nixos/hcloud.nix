{
  config,
  pkgs,
  lib,
  ...
}: {
  # The jktr template will populate config.hcloud.* from metadata
  # This file can be empty or contain hcloud-specific overrides

  # Example: Override SSH keys (if needed)
  # users.users.root.openssh.authorizedKeys.keys =
  #   lib.mkForce config.hcloud.metadata.public-keys;
}
