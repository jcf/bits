{
  config,
  pkgs,
  ...
}: let
  root = config.devenv.root;

  dev = {
    upstreams = {
      page = {port = 3000;};
      www = {port = 3100;};
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
        pattern = "~^(?<tenant>.+)\\.bits\\.page\\.test$";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.bits.page.test.pem";
        certKey = "${root}/certs/_wildcard.bits.page.test-key.pem";
      };

      www = {
        domain = "www.usebits.app.test";
        upstream = "www";
        certPem = "${root}/certs/_wildcard.usebits.app.test.pem";
        certKey = "${root}/certs/_wildcard.usebits.app.test-key.pem";
      };

      custom-domains = {
        pattern = "~^(?<custom_domain>.+\\.test)$";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.test.pem";
        certKey = "${root}/certs/_wildcard.test-key.pem";
      };
    };
  };
in {
  overlays = [
    (import ./nix/overlays/dioxus-cli.nix)
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
          DEVENV_ROOT = root;
        };
      };
    };
  };

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    DANGEROUSLY_ALLOW_JAVASCRIPT_EVALUATION = "true";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATABASE_URL_TEST = "postgres://bits:please@127.0.0.1:5432/bits_test";
    DOMAIN_PAGE = dev.hosts.page.domain;
    DOMAIN_WWW = dev.hosts.www.domain;
    MASTER_KEY = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    PLATFORM_DOMAIN = dev.hosts.page.domain;

    # TODO Create a dedicated Bits Postmark account.
    POSTMARK_ACCOUNT_TOKEN = "op://Invetica/Postmark/bits/account-api-token";
  };

  packages = with pkgs; [
    # Rust
    cargo-audit
    cargo-deny
    cargo-edit
    cargo-nextest
    dioxus-cli
    sqlx-cli
    wasm-bindgen-cli

    # Development
    fd
    just
    tokei
    tree
    zsh

    # SSL
    mkcert
    nss.tools

    # Formatters
    alejandra
    prettier
    shfmt
    taplo
    treefmt

    # Scraping
    firefox
    geckodriver
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;
  languages.javascript.pnpm.install.enable = true;

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  processes.www = {
    exec = "just www";
    process-compose.is_tty = true;
  };

  processes.tailwind-colo = {
    exec = "just tailwind crates/bits-colo";
  };

  processes.tailwind-solo = {
    exec = "just tailwind crates/bits-solo";
  };

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  process.managers.process-compose.settings.processes = {
    www = {
      environment = [
        "ASTRO_SITE=https://${dev.hosts.www.domain}"
        "PORT=${toString dev.upstreams.www.port}"
      ];
    };
  };

  services.nginx = {
    enable = true;
    httpConfig = ''
      error_log stderr error;

      upstream page {
        server localhost:${toString dev.upstreams.page.port};
      }

      upstream www {
        server localhost:${toString dev.upstreams.www.port};
      }

      # ${dev.hosts.page.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.page.domain};

        ssl_certificate ${dev.hosts.page.certPem};
        ssl_certificate_key ${dev.hosts.page.certKey};

        location / {
          proxy_pass http://${dev.hosts.page.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # ${dev.hosts.page-customers.pattern}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.page-customers.pattern};

        ssl_certificate ${dev.hosts.page-customers.certPem};
        ssl_certificate_key ${dev.hosts.page-customers.certKey};

        location / {
          proxy_pass http://${dev.hosts.page-customers.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # ${dev.hosts.custom-domains.pattern}
      server {
        listen 443 ssl default_server;
        server_name ~^(?<custom_domain>(?!.*\.(bits\.page|usebits\.app)\.test$).+\.test)$;

        ssl_certificate ${dev.hosts.custom-domains.certPem};
        ssl_certificate_key ${dev.hosts.custom-domains.certKey};

        location / {
          proxy_pass http://${dev.hosts.custom-domains.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # ${dev.hosts.www.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.www.domain};

        ssl_certificate ${dev.hosts.www.certPem};
        ssl_certificate_key ${dev.hosts.www.certKey};

        location / {
          proxy_pass http://${dev.hosts.www.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }
    '';
  };

  services.postgres = {
    enable = true;

    extensions = extensions: [
      extensions.pgvector
      extensions.postgis
    ];

    package = pkgs.postgresql_17;

    listen_addresses = "127.0.0.1";
    initialDatabases = [
      {
        name = "bits_dev";
        user = "bits";
        pass = "please";
      }
      {
        name = "bits_test";
        user = "bits";
        pass = "please";
      }
    ];

    initialScript = ''
      ALTER USER bits CREATEDB;
    '';
  };
}
