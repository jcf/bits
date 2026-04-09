{
  container-base,
  nix2container,
  pkgs,
}: let
  jaeger = pkgs.callPackage ../jaeger {};

  files = pkgs.runCommand "jaeger-files" {} ''
    mkdir -p $out/app/bin
    cp ${jaeger}/bin/jaeger $out/app/bin/
  '';

  rootLayer = pkgs.buildEnv {
    name = "jaeger-root";
    paths = container-base.paths ++ [files];
  };
in
  nix2container.buildImage {
    name = "bits-jaeger";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-jaeger" "Jaeger distributed tracing";

      Entrypoint = ["/app/bin/jaeger"];

      Env = [
        "LD_LIBRARY_PATH=/lib"
        "PATH=/app/bin"
      ];

      ExposedPorts = {
        "16686/tcp" = {};
        "4317/tcp" = {};
      };

      User = "${container-base.uid}:${container-base.uid}";
      WorkingDir = "/app";
    };
  }
