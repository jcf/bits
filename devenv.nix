{
  config,
  lib,
  pkgs,
  ...
}: let
  root = config.devenv.root;

  dev = {
    upstreams = {
      page = {port = 3030;};
      edit = {port = 3060;};
      www = {port = 3090;};
    };

    hosts = {
      page = {
        domain = "bits.page.test";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.page.test.pem";
        certKey = "${root}/certs/_wildcard.page.test-key.pem";
      };

      page-customers = {
        domain = "bits.page.test";
        pattern = "~^(?<customer>.+)\\.bits\\.page\\.test$";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.bits.page.test.pem";
        certKey = "${root}/certs/_wildcard.bits.page.test-key.pem";
        extraHeaders = ''
          proxy_set_header X-Customer $customer;
        '';
      };

      edit = {
        domain = "edit.usebits.app.test";
        upstream = "edit";
        certPem = "${root}/certs/_wildcard.usebits.app.test.pem";
        certKey = "${root}/certs/_wildcard.usebits.app.test-key.pem";
      };

      www = {
        domain = "www.usebits.app.test";
        upstream = "www";
        certPem = "${root}/certs/_wildcard.usebits.app.test.pem";
        certKey = "${root}/certs/_wildcard.usebits.app.test-key.pem";
      };
    };
  };
in {
  claude.code = {
    enable = true;

    mcpServers = {
      devenv = {
        type = "stdio";
        command = "devenv";
        args = ["mcp"];
        env = {
          DEVENV_ROOT = root;
        };
      };

      svelte = {
        type = "stdio";
        command = "svelte";
      };
    };
  };

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    DATABASE_URL = "op://Bits/neon/password";
    DOMAIN_EDIT = dev.hosts.edit.domain;
    DOMAIN_PAGE = dev.hosts.page.domain;
    DOMAIN_WWW = dev.hosts.www.domain;
  };

  overlays = [
    (import ./nix/overlays/wasm-bindgen-cli.nix)
  ];

  packages = with pkgs; [
    # Development
    cargo-audit
    cargo-deny
    cargo-edit
    cargo-nextest
    dioxus-cli
    fastly
    fd
    just
    just
    ldns # drill
    postgresql
    tokei
    wasm-bindgen-cli
    zsh

    # SSL
    mkcert
    nss.tools

    # Formatting
    alejandra
    prettier
    shfmt
    taplo
    treefmt

    # Build
    openssl
    pkg-config
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  # Dioxus requires its interactive TUI to work properly.
  processes.tailwind.exec = "pnpm tailwind:watch";

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;
}
