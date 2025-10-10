{
  pkgs,
  config,
  ...
}: let
  dev = {
    ports = {
      edit = 3060;
      page = 3030;
      www = 3000;
    };
  };
in {
  claude.code.enable = true;

  env = {
    DOMAIN_WWW = "land.invetica.dev";
    DOMAIN_EDIT = "edit.invetica.dev";
    DOMAIN_PAGE = "page.invetica.dev";
  };

  packages = with pkgs; [
    # Development
    fastly
    fd
    just
    postgresql
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

  # Nginx reverse proxy
  services.nginx = {
    enable = true;
    httpConfig = ''
      error_log stderr error;

      upstream edit {
        server localhost:${toString dev.ports.edit};
      }

      upstream page {
        server localhost:${toString dev.ports.page};
      }

      upstream www {
        server localhost:${toString dev.ports.www};
      }

      # Edit app
      server {
        listen 443 ssl;
        server_name edit.invetica.dev;

        ssl_certificate ${config.env.DEVENV_ROOT}/certs/_wildcard.invetica.dev.pem;
        ssl_certificate_key ${config.env.DEVENV_ROOT}/certs/_wildcard.invetica.dev-key.pem;

        location / {
          proxy_pass http://edit;
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # Page app
      server {
        listen 443 ssl;
        server_name page.invetica.dev;

        ssl_certificate ${config.env.DEVENV_ROOT}/certs/_wildcard.invetica.dev.pem;
        ssl_certificate_key ${config.env.DEVENV_ROOT}/certs/_wildcard.invetica.dev-key.pem;

        location / {
          proxy_pass http://page;
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # Customer subdomains (*.page.invetica.dev)
      server {
        listen 443 ssl;
        server_name ~^(?<customer>.+)\.page\.invetica\.dev$;

        ssl_certificate ${config.env.DEVENV_ROOT}/certs/_wildcard.page.invetica.dev.pem;
        ssl_certificate_key ${config.env.DEVENV_ROOT}/certs/_wildcard.page.invetica.dev-key.pem;

        location / {
          proxy_pass http://page;
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
          proxy_set_header X-Customer $customer;
        }
      }

      # `www` is not a loopback address so we use `land`.
      #
      # ➜ drill www.invetica.dev | rg '^www'
      # www.invetica.dev.       274     IN      A       172.67.178.210
      # www.invetica.dev.       274     IN      A       104.21.48.58
      server {
        listen 443 ssl;
        server_name land.invetica.dev;

        ssl_certificate ${config.env.DEVENV_ROOT}/certs/_wildcard.invetica.dev.pem;
        ssl_certificate_key ${config.env.DEVENV_ROOT}/certs/_wildcard.invetica.dev-key.pem;

        location / {
          proxy_pass http://www;
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
        "ASTRO_SITE=https://edit.invetica.dev"
        "PORT=${toString dev.ports.edit}"
      ];
    };
    page = {
      environment = [
        "ASTRO_SITE=https://page.invetica.dev"
        "PORT=${toString dev.ports.page}"
      ];
    };
    www = {
      environment = [
        "ASTRO_SITE=https://land.invetica.dev"
        "PORT=${toString dev.ports.www}"
      ];
    };
  };
}
