{
  container-base,
  nix2container,
  pkgs,
  traefik-static-config,
}: let
  inherit (pkgs) cacert traefik writeTextDir;

  config = writeTextDir "etc/traefik/traefik.yml"
    (builtins.readFile traefik-static-config);

  files = pkgs.runCommand "traefik-files" {} ''
    mkdir -p $out/app/bin
    cp ${traefik}/bin/traefik $out/app/bin/
  '';

  rootLayer = pkgs.buildEnv {
    name = "traefik-root";
    paths = container-base.paths ++ [cacert config files];
  };
in
  nix2container.buildImage {
    name = "bits-traefik";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-traefik" "Traefik reverse proxy";

      Entrypoint = ["/app/bin/traefik" "--configFile=/etc/traefik/traefik.yml"];

      Env = [
        "LD_LIBRARY_PATH=/lib"
        "PATH=/app/bin"
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
      ];

      ExposedPorts."443/tcp" = {};
      User = "${container-base.uid}:${container-base.uid}";
      WorkingDir = "/app";
    };
  }
