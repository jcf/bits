{pkgs, ...}: {
  packages = with pkgs; [
    # Rust development (these work with Nix-managed Rust)
    cargo-watch
    cargo-nextest
    cargo-audit
    cargo-deny
    cargo-edit

    # Smart contract development
    foundry # and not foundry-bin as the latter does not exist!

    # Infrastructure
    terraform
    _1password-cli

    # P2P/IPFS tools
    # ipfs # Temporarily disabled due to hash mismatch

    # Database tools
    sqlite
    sqlx-cli
  ];

  # Environment variables
  env.RUST_LOG = "debug";
  env.RUST_BACKTRACE = "1";
  env.DATABASE_URL = "sqlite:node/data/node.db";

  # Enable Rust with stable toolchain (Nix-managed)
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  # Keep it simple -- disable TUI of process-compose.
  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  processes = {
    # Main node process
    node = {
      exec = ''
        cargo watch -x "run --bin bits -- --dev"
      '';
    };
  };

  # Note: Development scripts have been moved to bin/ directory
  # Run 'bin/help' to see available commands

  # git-hooks and NOT pre-commit -- the latter is deprecated.
  git-hooks.hooks = {
    rustfmt = {
      enable = true;
      entry = "${pkgs.rustfmt}/bin/rustfmt --edition 2021 --check";
      files = "\.rs$";
      pass_filenames = true;
    };

    clippy = {
      enable = false; # Disabled for now as it's slow on pre-commit
    };
  };

  # Use cachix for faster builds
  cachix.enable = false;
}
