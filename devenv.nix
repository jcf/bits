{
  config,
  lib,
  pkgs,
  ...
}: let
  # Shared CI packages (keep in sync with bits-ci container)
  ci = import ./nix/ci.nix {inherit pkgs;};
  jdk = ci.jdk;

  # Local packages
  datomic-pro = pkgs.callPackage ./pkgs/datomic-pro {};
  forgejo-cli-ex = pkgs.callPackage ./pkgs/forgejo-cli-ex {};
  otel-agent = pkgs.callPackage ./pkgs/opentelemetry-javaagent {};
in {
  imports = [
    ./nix/modules/brotli.nix
    ./nix/modules/claude-code.nix
  ];

  profiles = {
    hostname."compute".module = {
      env.PG_CONN_STR = "postgres:///terraform_bits?host=/run/postgresql";
    };
    hostname."max".module = {
      env.PG_CONN_STR = "postgres://max@compute:5432/terraform_bits?sslmode=verify-full";
    };
    hostname."mini".module = {
      env.PG_CONN_STR = "postgres://mini@compute:5432/terraform_bits?sslmode=verify-full";
    };
  };

  cachix.enable = false;

  tasks."test:clojure" = {
    exec = "clojure -M:test:runner:linux-x86_64";
    before = ["devenv:enterTest"];
  };

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    CLUSTER_KEYSTORE_PASSWORD = "correct-horse-battery-staple";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATOMIC_URI = "datomic:sql://bits?jdbc:postgresql://127.0.0.1:5432/datomic?user=datomic&password=datomic";
    DOMAIN_PAGE = "bits.page.localhost";
    OTEL_JAVAAGENT_PATH = "${otel-agent}/lib/opentelemetry-javaagent.jar";
    PLATFORM_DOMAIN = "bits.page.localhost";
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
    otel-agent

    # Development
    fd
    getent
    forgejo-cli-ex
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
}
