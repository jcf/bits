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

  packages = with pkgs; [
    # Development
    fastly
    fd
    just
    ldns # drill
    postgresql
    tokei
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
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;

  languages.rust.enable = true;

  # Nginx reverse proxy
  services.nginx = {
    enable = true;
    httpConfig = ''
      error_log stderr error;

      upstream edit {
        server localhost:${toString dev.upstreams.edit.port};
      }

      upstream page {
        server localhost:${toString dev.upstreams.page.port};
      }

      upstream www {
        server localhost:${toString dev.upstreams.www.port};
      }

      # ${dev.hosts.edit.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.edit.domain};

        ssl_certificate ${dev.hosts.edit.certPem};
        ssl_certificate_key ${dev.hosts.edit.certKey};

        location / {
          proxy_pass http://${dev.hosts.edit.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
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
          ${dev.hosts.page-customers.extraHeaders}
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

  processes.edit.exec = "pnpm dev:edit";
  processes.page.exec = "pnpm dev:page";
  processes.www.exec = "pnpm dev:www";

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;
  process.managers.process-compose.settings.processes = {
    edit = {
      environment = [
        "ASTRO_SITE=https://${dev.hosts.edit.domain}"
        "PORT=${toString dev.upstreams.edit.port}"
      ];
    };
    page = {
      environment = [
        "ASTRO_SITE=https://${dev.hosts.page.domain}"
        "PORT=${toString dev.upstreams.page.port}"
      ];
    };
    www = {
      environment = [
        "ASTRO_SITE=https://${dev.hosts.www.domain}"
        "PORT=${toString dev.upstreams.www.port}"
      ];
    };
  };
}
