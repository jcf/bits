{
  nix2container,
  pkgs,
  error-pages-dir,
  fonts-dir,
}: let
  inherit (pkgs) buildEnv darkhttpd glibc runCommand stdenv;

  appDir = "app";
  wwwDir = "www";

  appLayer = buildEnv {
    name = "error-pages-app";
    paths = [
      (runCommand "error-pages-files" {} ''
        mkdir -p $out/${appDir}/bin $out/${wwwDir}/fonts $out/tmp
        chmod 1777 $out/tmp

        cp ${darkhttpd}/bin/darkhttpd $out/${appDir}/bin/

        # Error page HTML
        cp ${error-pages-dir}/502.html $out/${wwwDir}/

        # Fonts referenced by the error page
        cp ${fonts-dir}/DMSans.woff2 $out/${wwwDir}/fonts/ 2>/dev/null || true
        cp ${fonts-dir}/JetBrainsMono.woff2 $out/${wwwDir}/fonts/ 2>/dev/null || true
      '')
    ];
  };

  libsLayer = buildEnv {
    name = "error-pages-libs";
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
    name = "bits-error-pages";

    copyToRoot = [libsLayer appLayer];

    config = {
      Labels = {
        "org.opencontainers.image.description" = "Error pages for Traefik";
        "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
        "org.opencontainers.image.title" = "bits-error-pages";
      };

      Entrypoint = [
        "/${appDir}/bin/darkhttpd"
        "/${wwwDir}"
        "--port"
        "8080"
        "--no-listing"
      ];

      Env = [
        "LD_LIBRARY_PATH=/lib"
      ];

      ExposedPorts."8080/tcp" = {};
      User = "1000:1000";
      WorkingDir = "/${wwwDir}";
    };
  }
