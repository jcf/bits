{
  description = "Bits - Multi-tenant web platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    ragenix.url = "github:yaxitech/ragenix";
    ragenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ragenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};

        # Read version from workspace Cargo.toml
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;

        # Get git revision for build metadata
        gitRev = self.rev or self.dirtyRev or "dirty";
        gitShortRev = builtins.substring 0 7 gitRev;

        # Build the bits binary
        bits = pkgs.rustPlatform.buildRustPackage {
          pname = "bits";
          version = "${version}+${gitShortRev}";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs;
            [
              openssl
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
              SystemConfiguration
            ]);

          # Build only the bits-colo binary with server and colo features
          buildAndTestSubdir = "crates/bits-colo";
          cargoBuildFlags = ["--package" "bits-colo"];

          # Skip tests during build (run separately in CI)
          doCheck = false;

          meta = with pkgs.lib; {
            description = "Bits - Multi-tenant web platform";
            homepage = "https://bits.page";
            license = licenses.agpl3Plus;
            maintainers = [];
          };
        };

        # Build Dioxus frontend bundle
        bits-bundle = pkgs.stdenv.mkDerivation {
          pname = "bits-bundle";
          version = "${version}+${gitShortRev}";

          src = ./.;

          nativeBuildInputs = with pkgs; [
            nodejs
            nodePackages.pnpm
            cargo
            rustc
          ];

          buildPhase = ''
            # Install JS dependencies
            pnpm install --frozen-lockfile

            # Build Tailwind CSS
            pnpm --filter @bits/tailwind exec tailwindcss \
              --input crates/bits-colo/tailwind.css \
              --output crates/bits-colo/assets/app.css

            # Build Dioxus bundle
            export HOME=$TMPDIR
            dx bundle --release --package bits-colo
          '';

          installPhase = ''
            mkdir -p $out
            cp -r target/dx/bits-colo/release/public/* $out/
          '';
        };
      in {
        packages = {
          inherit bits bits-bundle;
          default = bits;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [bits];
          buildInputs = with pkgs; [
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    )
    // {
      # NixOS module for bits-platform systemd service
      nixosModules.bits-platform = {
        config,
        lib,
        pkgs,
        ...
      }: let
        cfg = config.services.bits-platform;
        bits = self.packages.${pkgs.system}.bits;
        bits-bundle = self.packages.${pkgs.system}.bits-bundle;
      in {
        options.services.bits-platform = {
          enable = lib.mkEnableOption "Bits platform service";

          databaseUrlFile = lib.mkOption {
            type = lib.types.path;
            description = "Path to file containing DATABASE_URL";
          };

          platformDomain = lib.mkOption {
            type = lib.types.str;
            default = "bits.page";
            description = "Platform domain name";
          };

          masterKeyFile = lib.mkOption {
            type = lib.types.path;
            description = "Path to file containing MASTER_KEY";
          };

          port = lib.mkOption {
            type = lib.types.port;
            default = 8080;
            description = "Port to listen on";
          };
        };

        config = lib.mkIf cfg.enable {
          systemd.services.bits-platform = {
            description = "Bits Platform (co-located)";
            wantedBy = ["multi-user.target"];
            after = ["network.target"];

            # Run migrations before starting service
            preStart = ''
              echo "Running database migrations..."
              ${bits}/bin/bits migrate \
                --database-url "$(cat ${cfg.databaseUrlFile})"
            '';

            serviceConfig = {
              Type = "simple";
              ExecStart = "${bits}/bin/bits serve";
              Restart = "on-failure";
              RestartSec = "5s";

              # Health check after startup
              ExecStartPost = "${pkgs.bash}/bin/bash -c 'sleep 2 && ${pkgs.curl}/bin/curl -f http://localhost:${toString cfg.port}/healthz'";

              # Environment
              Environment = [
                "PLATFORM_DOMAIN=${cfg.platformDomain}"
                "PUBLIC_DIR=${bits-bundle}"
                "PORT=${toString cfg.port}"
              ];

              # Load secrets from files
              EnvironmentFile = [
                cfg.databaseUrlFile
                cfg.masterKeyFile
              ];

              # Security hardening
              DynamicUser = true;
              NoNewPrivileges = true;
              PrivateTmp = true;
              ProtectSystem = "strict";
              ProtectHome = true;
              ProtectKernelTunables = true;
              ProtectKernelModules = true;
              ProtectControlGroups = true;
              RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
              RestrictNamespaces = true;
              LockPersonality = true;
              RestrictRealtime = true;
              RestrictSUIDSGID = true;
              PrivateDevices = true;
              SystemCallArchitectures = "native";
            };
          };

          # Open firewall for the application port
          networking.firewall.allowedTCPPorts = [cfg.port];
        };
      };

      # NixOS configurations for deployment
      nixosConfigurations = let
        mkConfiguration = {
          hostname,
          environment,
        }:
          nixpkgs.lib.nixosSystem {
            system = "x86_64-linux";
            modules = [
              ./nix/nixos/configuration.nix
              ragenix.nixosModules.default
              self.nixosModules.bits-platform
              ({config, ...}: {
                networking.hostName = hostname;

                services.bits-platform = {
                  enable = true;
                  databaseUrlFile = config.age.secrets.database_url.path;
                  masterKeyFile = config.age.secrets.master_key.path;
                  platformDomain =
                    if environment == "prod"
                    then "bits.page"
                    else "bits.page.test";
                };

                age.secrets = {
                  database_url = {
                    file = ./secrets/database_url_${environment}.age;
                    mode = "0440";
                  };
                  master_key = {
                    file = ./secrets/master_key_${environment}.age;
                    mode = "0440";
                  };
                };
              })
            ];
          };
      in {
        bits-prod = mkConfiguration {
          hostname = "bits-prod";
          environment = "prod";
        };

        bits-dev = mkConfiguration {
          hostname = "bits-dev";
          environment = "dev";
        };
      };
    };
}
