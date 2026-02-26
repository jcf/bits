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

      # Container builder for a Linux system
      mkContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
        n2cLinux = nix2container.packages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/bits-container {
          inherit otel-agent;
          inherit (n2cLinux) nix2container;
          otel-agent-properties = ./resources/otel-agent.properties;
          uberjar = bits-uberjar;
        };

      # CI container builder
      mkCiContainer = targetSystem: let
        pkgsLinux = nixpkgs.legacyPackages.${targetSystem};
        n2cLinux = nix2container.packages.${targetSystem};
        cljNixLinux = clj-nix.packages.${targetSystem};
      in
        pkgsLinux.callPackage ./pkgs/bits-ci {
          inherit (n2cLinux) nix2container;
          inherit (cljNixLinux) deps-lock;
        };
    in {
      # CI container (amd64 only)
      bits-ci = mkCiContainer "x86_64-linux";

      # Application containers
      bits-container-amd64 = mkContainer "x86_64-linux";
      bits-container-arm64 = mkContainer "aarch64-linux";

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
