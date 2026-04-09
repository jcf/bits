{
  datomic-pro,
  envsubst,
  lib,
  nix2container,
  pkgs,
  writeTextDir,
}: let
  inherit (pkgs) buildEnv busybox glibc runCommand stdenv;

  # Container paths
  appDir = "app";

  # Properties template - envsubst replaces $VARS at container startup
  propertiesTemplate = writeTextDir "${appDir}/transactor.properties.template" ''
    protocol=sql
    host=0.0.0.0
    port=4334
    alt-host=$ALT_HOST

    # PostgreSQL storage
    sql-url=$SQL_URL
    sql-driver-class=org.postgresql.Driver

    # Memory settings (required for SQL protocol)
    memory-index-max=$DATOMIC_MEMORY_INDEX_MAX
    memory-index-threshold=32m
    object-cache-max=$DATOMIC_OBJECT_CACHE_MAX
  '';

  # Application layer - copy everything to /app to avoid /bin conflicts
  appLayer = buildEnv {
    name = "datomic-transactor-app";
    paths = [
      propertiesTemplate
      (runCommand "datomic-transactor-files" {} ''
        mkdir -p $out/${appDir}/bin $out/bin $out/tmp
        chmod 1777 $out/tmp

        # Shell for startup script
        ln -s ${busybox}/bin/sh $out/bin/sh

        # Copy datomic-pro bin directory contents
        cp -r ${datomic-pro}/bin/* $out/${appDir}/bin/
        cp -r ${datomic-pro}/lib $out/${appDir}/
        cp -r ${datomic-pro}/share $out/${appDir}/

        # Copy envsubst
        cp ${envsubst}/bin/envsubst $out/${appDir}/bin/

        # Create startup script
        cat > $out/${appDir}/bin/start-transactor << 'EOF'
        #!/bin/sh
        set -eu
        /app/bin/envsubst < /app/transactor.properties.template > /tmp/transactor.properties
        exec /app/bin/datomic-transactor /tmp/transactor.properties
        EOF
        chmod +x $out/${appDir}/bin/start-transactor
      '')
    ];
  };

  # System libraries layer - only libs, no /bin
  libsLayer = buildEnv {
    name = "datomic-transactor-libs";
    paths = [
      (runCommand "libs" {} ''
        mkdir -p $out/lib

        # Copy glibc libs (not bin)
        cp -r ${glibc}/lib/* $out/lib/

        # Copy libstdc++ etc
        cp -r ${stdenv.cc.cc.lib}/lib/* $out/lib/

        # Dynamic linker symlink
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
    name = "datomic-transactor";

    copyToRoot = [libsLayer appLayer];

    config = {
      Labels = {
        "org.opencontainers.image.description" = "Datomic Pro transactor";
        "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
        "org.opencontainers.image.title" = "datomic-transactor";
      };

      Entrypoint = ["/${appDir}/bin/start-transactor"];

      Env = [
        "PATH=/${appDir}/bin"
        "LD_LIBRARY_PATH=/lib"
      ];

      ExposedPorts."4334/tcp" = {};
      User = "1000:1000";
      WorkingDir = "/${appDir}";
    };
  }
