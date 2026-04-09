{
  nix2container,
  pkgs,
}: let
  inherit (pkgs) buildEnv glibc runCommand stdenv;

  jaeger = pkgs.callPackage ../jaeger {};

  appDir = "app";

  appLayer = buildEnv {
    name = "jaeger-app";
    paths = [
      (runCommand "jaeger-files" {} ''
        mkdir -p $out/${appDir}/bin $out/tmp
        chmod 1777 $out/tmp
        cp ${jaeger}/bin/jaeger $out/${appDir}/bin/
      '')
    ];
  };

  libsLayer = buildEnv {
    name = "jaeger-libs";
    paths = [
      (runCommand "libs" {} ''
        mkdir -p $out/lib
        cp -r ${glibc}/lib/* $out/lib/
        cp -r ${stdenv.cc.cc.lib}/lib/* $out/lib/

        ${
          if stdenv.hostPlatform.isAarch64
          then "ln -s /lib/ld-linux-aarch64.so.1 $out/lib/ld-linux-aarch64.so.1 2>/dev/null || true"
          else ''
            mkdir -p $out/lib64
            ln -s /lib/ld-linux-x86-64.so.2 $out/lib64/ld-linux-x86-64.so.2
          ''
        }
      '')
    ];
  };
in
  nix2container.buildImage {
    name = "bits-jaeger";

    copyToRoot = [libsLayer appLayer];

    config = {
      Labels = {
        "org.opencontainers.image.description" = "Jaeger distributed tracing";
        "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
        "org.opencontainers.image.title" = "bits-jaeger";
      };

      Entrypoint = ["/${appDir}/bin/jaeger"];

      Env = [
        "PATH=/${appDir}/bin"
        "LD_LIBRARY_PATH=/lib"
      ];

      ExposedPorts = {
        "4317/tcp" = {};
        "16686/tcp" = {};
      };

      User = "1000:1000";
      WorkingDir = "/${appDir}";
    };
  }
