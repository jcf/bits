{
  container-base,
  nix2container,
  pkgs,
}: let
  inherit (pkgs) cacert traefik;

  files = pkgs.runCommand "traefik-files" {} ''
    mkdir -p $out/app/bin $out/etc/traefik/dynamic
    cp ${traefik}/bin/traefik $out/app/bin/
  '';

  rootLayer = pkgs.buildEnv {
    name = "traefik-root";
    paths = container-base.paths ++ [cacert files];
  };
in
  nix2container.buildImage {
    name = "bits-traefik";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-traefik" "Traefik reverse proxy";

      Entrypoint = ["/app/bin/traefik"];

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
