{
  description = "Bits";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    nix2container = {
      url = "git+https://code.invetica.team/jcf/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    clj-nix = {
      url = "git+https://code.invetica.team/jcf/clj-nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nix-fetcher-data.follows = "nix-fetcher-data";
    };

    nix-fetcher-data = {
      url = "git+https://code.invetica.team/jcf/nix-fetcher-data";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Override in CI: --override-input deps-lock path:/tmp/deps-lock.json
    deps-lock = {
      url = "path:./deps-lock.json";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    nix2container,
    clj-nix,
    deps-lock,
    ...
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

    # Build packages for a specific system
    mkPackages = system: let
      pkgs = nixpkgs.legacyPackages.${system};
      cljNix = clj-nix.packages.${system};

      # Uberjar built from deps-lock.json
      bits-uberjar = pkgs.callPackage ./pkgs/bits-uberjar {
        inherit (cljNix) fake-git mk-deps-cache;
        jdk = pkgs.temurin-bin-21;
        lockfile = deps-lock;
      };

      otel-agent = pkgs.callPackage ./pkgs/opentelemetry-javaagent {};

      # nix2container for the HOST system — copyTo must run on macOS
      n2cHost = nix2container.packages.${system};

      # Container builder for a Linux system
      mkContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/bits-container {
          inherit otel-agent;
          inherit (n2cHost) nix2container;
          otel-agent-properties = ./resources/otel-agent.properties;
          uberjar = bits-uberjar;
        };

      # CI container builder
      mkCiContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
        cljNixLinux = clj-nix.packages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/bits-ci {
          inherit (n2cHost) nix2container;
          inherit (cljNixLinux) deps-lock;
        };

      # Transactor container builder
      mkTransactorContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/datomic-transactor-container {
          inherit (n2cHost) nix2container;
          datomic-pro = pkgsLinux.callPackage ./pkgs/datomic-pro {};
        };

      # Jaeger container builder
      mkJaegerContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/jaeger-container {
          inherit (n2cHost) nix2container;
        };

      # Traefik container builder
      mkTraefikContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/traefik-container {
          inherit (n2cHost) nix2container;
          traefik-static-config = ./docker/traefik/traefik.yml;
        };

      # Error pages container builder
      mkErrorPagesContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/error-pages-container {
          inherit (n2cHost) nix2container;
          error-pages-dir = ./nix/nginx;
          fonts-dir = ./resources/public;
        };

      # PostgreSQL container builder
      mkPostgresContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/postgres-container {
          inherit (n2cHost) nix2container;
          init-sql = ./docker/postgres/init.sql;
        };

      # Dev container builder
      mkDevContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/bits-dev-container {
          inherit (n2cHost) nix2container;
          container-base = pkgsLinux.callPackage ./pkgs/container-base {};
          otel-agent = pkgsLinux.callPackage ./pkgs/opentelemetry-javaagent {};
        };

      # Tailwind container builder
      mkTailwindContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/tailwind-container {
          inherit (n2cHost) nix2container;
          container-base = pkgsLinux.callPackage ./pkgs/container-base {};
        };
    in {
      # CI container (amd64 only)
      bits-ci = mkCiContainer "x86_64-linux";

      # Application containers
      bits-container-amd64 = mkContainer "x86_64-linux";
      bits-container-arm64 = mkContainer "aarch64-linux";

      # Transactor containers
      transactor-container-amd64 = mkTransactorContainer "x86_64-linux";
      transactor-container-arm64 = mkTransactorContainer "aarch64-linux";

      # Dev infrastructure containers (arm64 for local dev)
      jaeger-container-arm64 = mkJaegerContainer "aarch64-linux";
      traefik-container-arm64 = mkTraefikContainer "aarch64-linux";
      error-pages-container-arm64 = mkErrorPagesContainer "aarch64-linux";
      postgres-container-arm64 = mkPostgresContainer "aarch64-linux";
      dev-container-arm64 = mkDevContainer "aarch64-linux";
      tailwind-container-arm64 = mkTailwindContainer "aarch64-linux";

      # Uberjar
      bits-uberjar = bits-uberjar;
      bits-deps-cache = bits-uberjar.depsCache;

      # Other packages
      datomic-pro = pkgs.callPackage ./pkgs/datomic-pro {};
    };
  in {
    packages = forAllSystems mkPackages;
  };
}
