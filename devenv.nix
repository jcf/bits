{
  config,
  lib,
  inputs,
  pkgs,
  ...
}: let
  root = config.devenv.root;

  # Version from flake source info
  version = let
    # lastModifiedDate is YYYYMMDDHHMMSS format
    date = builtins.substring 0 8 (toString (inputs.self.lastModifiedDate or 0));
    # shortRev for clean (abc1234), dirtyShortRev for dirty (abc1234-dirty)
    rev = inputs.self.shortRev or inputs.self.dirtyShortRev or "dirty";
  in
    if date == "0"
    then "dirty"
    else "${date}-${rev}";

  # Shared CI packages (keep in sync with bits-ci container)
  ci = import ./nix/ci.nix {inherit pkgs;};

  jdk = ci.jdk;

  # Build uberjar with JDK 21 to avoid Clojure AOT + JDK 25 bytecode verification bug
  # (ClassFormatError: Invalid index in LocalVariableTable). Runtime is still JDK 25.
  jdk-build = pkgs.temurin-bin-21;

  # clj-nix for deterministic Clojure dependency management
  cljNix = inputs.clj-nix.packages.${pkgs.system};

  bits-uberjar = pkgs.callPackage ./pkgs/bits-uberjar {
    inherit version;
    inherit (cljNix) fake-git mk-deps-cache;
    jdk = jdk-build;
  };
  datomic-pro = pkgs.callPackage ./pkgs/datomic-pro {};
  jaeger = pkgs.callPackage ./pkgs/jaeger {};
  otel-agent = pkgs.callPackage ./pkgs/opentelemetry-javaagent {};

  # Container builder for a specific Linux system
  mkContainer = system:
    pkgs.callPackage ./pkgs/bits-container {
      inherit otel-agent version;
      pkgsLinux = import inputs.nixpkgs {inherit system;};
      otel-agent-properties = ./resources/otel-agent.properties;
      uberjar = bits-uberjar;
    };

  # Default container for local dev (Apple Silicon + OrbStack)
  bits-container = mkContainer "aarch64-linux";

  # CI container builder
  mkCiContainer = system:
    pkgs.callPackage ./pkgs/bits-ci {
      inherit version;
      pkgsLinux = import inputs.nixpkgs {inherit system;};
    };

  dev = {
    upstreams = {
      page = {port = 3000;};
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
    ./nix/modules/brotli.nix
    ./nix/modules/claude-code.nix
  ];

  cachix.enable = false;

  tasks."test:clojure" = {
    exec = "clojure -M:test:runner:linux-x86_64";
    before = ["devenv:enterTest"];
  };

  outputs.bits-ci-amd64 = mkCiContainer "x86_64-linux";
  outputs.bits-ci-arm64 = mkCiContainer "aarch64-linux";
  outputs.bits-container = bits-container;
  outputs.bits-container-amd64 = mkContainer "x86_64-linux";
  outputs.bits-container-arm64 = mkContainer "aarch64-linux";
  outputs.bits-uberjar = bits-uberjar;
  outputs.bits-deps-cache = bits-uberjar.depsCache;
  outputs.datomic-pro = datomic-pro;

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    CLUSTER_KEYSTORE_PASSWORD = "correct-horse-battery-staple";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATOMIC_URI = "datomic:sql://bits?jdbc:postgresql://127.0.0.1:5432/datomic?user=datomic&password=datomic";
    DOMAIN_PAGE = dev.hosts.page.domain;
    OTEL_JAVAAGENT_PATH = "${otel-agent}/lib/opentelemetry-javaagent.jar";
    PLATFORM_DOMAIN = dev.hosts.page.domain;
    SSE_RECONNECT_MS = "50";
  };

  packages = with pkgs; [
    # Clojure
    babashka
    clj-kondo
    cljfmt
    (clojure.override {jdk = jdk;})
    clojure-lsp
    jdk

    datomic-pro

    # Observability
    jaeger
    otel-agent

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
    tailwindcss_4
    taplo
    treefmt
  ];

  processes = lib.optionalAttrs (!config.devenv.isTesting) {
    nrepl = {
      exec = "just nrepl";
      process-compose.is_tty = true;
    };

    tailwind = {
      exec = "just tailwind";
      process-compose.is_tty = true;
    };

    transactor = {
      exec = "datomic-transactor conf/datomic.dev.properties";
      process-compose.is_tty = true;
    };

    jaeger = {
      exec = "jaeger";
      process-compose.is_tty = true;
    };
  };

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  process.managers.process-compose.settings.processes = {
    nrepl = {
      environment = [
        "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317"
        "OTEL_SERVICE_NAME=bits"
      ];
    };
    transactor = {
      depends_on.postgres.condition = "process_healthy";
    };
  };

  services.nginx = {
    enable = !config.devenv.isTesting;
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
        server_name ~^(?<custom_domain>(?!.*\.bits\.page\.test$).+\.test)$;

        ssl_certificate ${dev.hosts.custom-domains.certPem};
        ssl_certificate_key ${dev.hosts.custom-domains.certKey};

        ${errorLocations}

        location / {
          ${proxySettings dev.hosts.custom-domains.upstream}
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
      ALTER USER bits WITH PASSWORD 'please' CREATEDB;
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
