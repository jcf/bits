{
  config,
  pkgs,
  ...
}: {
  overlays = [
    (import ./nix/overlays/wasm-bindgen-cli.nix)
  ];

  claude.code = {
    enable = true;

    mcpServers = {
      devenv = {
        type = "stdio";
        command = "devenv";
        args = ["mcp"];
        env = {
          DEVENV_ROOT = config.devenv.root;
        };
      };
    };
  };

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
  };

  packages = with pkgs; [
    # Rust
    cargo-audit
    cargo-deny
    cargo-edit
    cargo-nextest
    dioxus-cli
    wasm-bindgen-cli

    # Development
    fd
    just
    tokei
    zsh

    # Formatters
    alejandra
    prettier
    shfmt
    taplo
    treefmt
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  processes.tailwind = {
    exec = "just css";
    process-compose = {
      is_tty = true;
    };
  };

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;
}
