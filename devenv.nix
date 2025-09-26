{
  pkgs,
  config,
  ...
}: {
  claude.code.enable = true;

  packages = with pkgs; [
    # Development
    fastly
    just

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
      upstream edit {
        server localhost:3000;
      }

      upstream page {
        server localhost:3030;
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
    '';
  };

  processes.edit.exec = "pnpm dev:edit";
  processes.page.exec = "pnpm dev:page";

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;
  process.managers.process-compose.settings.processes = {
    edit = {
      environment = [
        "ASTRO_SITE=https://edit.invetica.dev"
        "PORT=3000"
      ];
    };
    page = {
      environment = [
        "ASTRO_SITE=https://page.invetica.dev"
        "PORT=3030"
      ];
    };
  };
}
