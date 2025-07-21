{pkgs, ...}: {
  packages = with pkgs; [
    cargo-watch
    cargo-nextest
    cargo-audit
    cargo-deny
    cargo-edit

    # Smart contract development
    foundry

    # Database tools
    sqlite
    sqlx-cli
  ];

  env.DATABASE_URL = "sqlite:node/data/node.db";
  env.RUST_BACKTRACE = "1";
  env.RUST_LOG = "debug";

  languages.rust = {
    enable = true;
    channel = "stable";
  };

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  processes = {
    node = {
      exec = "cargo watch -x 'run --bin bits' -w crates/";
    };
  };
}
