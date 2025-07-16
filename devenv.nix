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
    ipfs

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
    node = {
      exec = ''
        cargo watch -x "run --bin bits -- --dev"
      '';
    };
  };

  # Development scripts
  scripts = {
    # Run the node
    dev.exec = ''
      cargo run --bin bits -- --dev
    '';

    # Run all tests
    test.exec = ''
      cargo nextest run
    '';

    # Build release binary
    build.exec = ''
      cargo build --release
    '';

    # Deploy contracts to local testnet
    deploy-local.exec = ''
      cd contracts && forge script Deploy --broadcast --rpc-url http://localhost:8545
    '';

    # Format code
    fmt.exec = ''
      cargo fmt --all
    '';

    # Lint code
    lint.exec = ''
      cargo clippy --all-targets --all-features -- -D warnings
    '';

    # Security audit
    audit.exec = ''
      cargo audit
      cargo deny check
    '';

    # Initialize SQLite database
    db-init.exec = ''
      mkdir -p node/data
      sqlx database create
      sqlx migrate run --source node/migrations
    '';

    # Generate new migration
    db-migrate.exec = ''
      cd node && sqlx migrate add $1
    '';

    # Clean build artifacts
    clean.exec = ''
      cargo clean
      rm -rf node/data
    '';
  };

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
