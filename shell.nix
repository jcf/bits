{pkgs}: let
  inherit (pkgs) lib stdenv;
  ci = import ./nix/ci.nix {inherit pkgs;};
  datomic-pro = pkgs.callPackage ./pkgs/datomic-pro {};
  forgejo-cli-ex = pkgs.callPackage ./pkgs/forgejo-cli-ex {};
in
  pkgs.mkShellNoCC {
    packages = with pkgs;
      [
        alejandra
        babashka
        ci.jdk
        cljfmt
        clj-kondo
        (clojure.override {jdk = ci.jdk;})
        clojure-lsp
        datomic-pro
        fd
        forgejo-cli-ex
        getent
        just
        mkcert
        nss.tools
        prettier
        shfmt
        taplo
        tokei
        tree
        treefmt
        zsh
      ]
      ++ lib.optionals stdenv.isLinux [stdenv.cc.cc.lib];

    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATOMIC_URI = "datomic:sql://bits?jdbc:postgresql://127.0.0.1:5432/datomic?user=datomic&password=datomic";
    DOMAIN_PAGE = "bits.page.localhost";
    PLATFORM_DOMAIN = "bits.page.localhost";

    shellHook = ''
      ${lib.optionalString stdenv.isLinux ''
        export LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib"
      ''}
      case "$(hostname -s 2>/dev/null || hostname)" in
        compute) export PG_CONN_STR="postgres:///terraform_bits?host=/run/postgresql" ;;
        max)     export PG_CONN_STR="postgres://max@compute:5432/terraform_bits?sslmode=verify-full" ;;
        mini)    export PG_CONN_STR="postgres://mini@compute:5432/terraform_bits?sslmode=verify-full" ;;
      esac

      unset shellHook buildPhase phases
    '';
  }
