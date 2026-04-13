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

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATOMIC_URI = "datomic:sql://bits?jdbc:postgresql://127.0.0.1:5432/datomic?user=datomic&password=datomic";
    DOMAIN_PAGE = "bits.page.localhost";
    PLATFORM_DOMAIN = "bits.page.localhost";
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

    # Development
    fd
    forgejo-cli-ex
    getent
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
  ];
}
