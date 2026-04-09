{
  nix2container,
  pkgs,
  traefik-static-config,
}: let
  inherit (pkgs) buildEnv cacert glibc runCommand stdenv traefik writeTextDir;

  appDir = "app";

  configLayer = buildEnv {
    name = "traefik-config";
    paths = [
      (writeTextDir "etc/traefik/traefik.yml"
        (builtins.readFile traefik-static-config))
    ];
  };

  appLayer = buildEnv {
    name = "traefik-app";
    paths = [
      (runCommand "traefik-files" {} ''
        mkdir -p $out/${appDir}/bin $out/tmp
        chmod 1777 $out/tmp
        cp ${traefik}/bin/traefik $out/${appDir}/bin/
      '')
    ];
  };

  libsLayer = buildEnv {
    name = "traefik-libs";
    paths = [
      cacert
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
    name = "bits-traefik";

    copyToRoot = [libsLayer configLayer appLayer];

    config = {
      Labels = {
        "org.opencontainers.image.description" = "Traefik reverse proxy";
        "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
        "org.opencontainers.image.title" = "bits-traefik";
      };

      Entrypoint = [
        "/${appDir}/bin/traefik"
        "--configFile=/etc/traefik/traefik.yml"
      ];

      Env = [
        "PATH=/${appDir}/bin"
        "LD_LIBRARY_PATH=/lib"
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
      ];

      ExposedPorts."443/tcp" = {};
      User = "1000:1000";
      WorkingDir = "/${appDir}";
    };
  }
