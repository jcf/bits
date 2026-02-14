{
  config,
  pkgs,
  ...
}: let
  root = config.devenv.root;
  datomic-pro = pkgs.callPackage ./pkgs/datomic-pro {};

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
  imports = [
    ./nix/modules/claude-code.nix
  ];

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATOMIC_URI = "datomic:sql://bits?jdbc:postgresql://127.0.0.1:5432/datomic?user=datomic&password=datomic";
    DOMAIN_PAGE = dev.hosts.page.domain;
    DOMAIN_WWW = dev.hosts.www.domain;
    PLATFORM_DOMAIN = dev.hosts.page.domain;
    SSE_RECONNECT_MS = "50";
  };

  packages = with pkgs; [
    # Clojure
    babashka
    clj-kondo
    cljfmt
    clojure
    clojure-lsp

    # Database
    datomic-pro

    # Development
    fd
    just
    tokei
    tree
    zsh

    # Browsers
    chromedriver
    geckodriver

    # SSL
    mkcert
    nss.tools

    # Formatters
    alejandra
    prettier
    shfmt
    taplo
    treefmt
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;
  languages.javascript.pnpm.install.enable = true;

  processes.nrepl = {
    exec = "just nrepl";
  };

  processes.market = {
    exec = "just market";
    process-compose.is_tty = true;
  };

  processes.tailwind = {
    exec = "just tailwind";
    process-compose.is_tty = true;
  };

  processes.transactor = {
    exec = "datomic-transactor conf/datomic.dev.properties";
    process-compose.is_tty = true;
  };

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  process.managers.process-compose.settings.processes = {
    transactor = {
      depends_on.postgres.condition = "process_healthy";
    };
    www = {
      environment = [
        "ASTRO_SITE=https://${dev.hosts.www.domain}"
        "PORT=${toString dev.upstreams.www.port}"
      ];
    };
  };

  services.nginx = {
    enable = true;
    httpConfig = let
      # Common proxy settings for SSE support
      proxySettings = upstream: ''
        proxy_pass http://${upstream};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # SSE support
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400s;
      '';

      # Error page and static asset locations
      errorLocations = ''
        error_page 502 503 504 /_nginx/502.html;

        location /_nginx/ {
          alias ${root}/nix/nginx/;
          internal;
        }

        location /_nginx/fonts/ {
          alias ${root}/resources/public/;
        }
      '';
    in ''
      error_log stderr error;

      upstream page {
        server localhost:${toString dev.upstreams.page.port} fail_timeout=0;
      }

      upstream www {
        server localhost:${toString dev.upstreams.www.port} fail_timeout=0;
      }

      # ${dev.hosts.page.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.page.domain};

        ssl_certificate ${dev.hosts.page.certPem};
        ssl_certificate_key ${dev.hosts.page.certKey};

        ${errorLocations}

        location / {
          ${proxySettings dev.hosts.page.upstream}
        }
      }

      # ${dev.hosts.page-customers.pattern}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.page-customers.pattern};

        ssl_certificate ${dev.hosts.page-customers.certPem};
        ssl_certificate_key ${dev.hosts.page-customers.certKey};

        ${errorLocations}

        location / {
          ${proxySettings dev.hosts.page-customers.upstream}
        }
      }

      # ${dev.hosts.custom-domains.pattern}
      server {
        listen 443 ssl default_server;
        server_name ~^(?<custom_domain>(?!.*\.(bits\.page|usebits\.app)\.test$).+\.test)$;

        ssl_certificate ${dev.hosts.custom-domains.certPem};
        ssl_certificate_key ${dev.hosts.custom-domains.certKey};

        ${errorLocations}

        location / {
          ${proxySettings dev.hosts.custom-domains.upstream}
        }
      }

      # ${dev.hosts.www.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.www.domain};

        ssl_certificate ${dev.hosts.www.certPem};
        ssl_certificate_key ${dev.hosts.www.certKey};

        ${errorLocations}

        location / {
          ${proxySettings dev.hosts.www.upstream}
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
      {
        name = "datomic";
        user = "datomic";
        pass = "datomic";
      }
    ];

    initialScript = ''
      ALTER USER bits CREATEDB;
      ALTER DATABASE bits_test OWNER TO bits;

      -- Datomic KV storage table
      \c datomic
      CREATE TABLE IF NOT EXISTS datomic_kvs (
        id text NOT NULL,
        rev integer,
        map text,
        val bytea,
        CONSTRAINT pk_id PRIMARY KEY (id)
      );
      ALTER TABLE datomic_kvs OWNER TO datomic;
      GRANT ALL ON TABLE datomic_kvs TO datomic;
    '';
  };
}
