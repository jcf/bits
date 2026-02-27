{
  datomic-pro,
  dockerTools,
  envsubst,
  lib,
  nix2container,
  pkgs,
  writeTextDir,
}: let
  inherit (pkgs) buildEnv glibc runCommand stdenv;

  # Container paths
  appDir = "app";

  # Properties template - envsubst replaces $VARS at container startup
  propertiesTemplate = writeTextDir "${appDir}/transactor.properties.template" ''
    protocol=sql
    host=0.0.0.0
    port=4334
    alt-host=bits-transactor

    # PostgreSQL storage
    sql-url=$SQL_URL
    sql-driver-class=org.postgresql.Driver

    # Memory settings
    memory-index-max=256m
    memory-index-threshold=32m
    object-cache-max=128m
  '';

  # Startup script that renders template and runs transactor
  startScript =
    runCommand "start-transactor" {
      nativeBuildInputs = [pkgs.makeWrapper];
    } ''
      mkdir -p $out/bin
      cat > $out/bin/start-transactor << 'EOF'
      #!/bin/sh
      set -eu
      envsubst < /app/transactor.properties.template > /tmp/transactor.properties
      exec datomic-transactor /tmp/transactor.properties
      EOF
      chmod +x $out/bin/start-transactor
    '';

  # Application layer
  appLayer = buildEnv {
    name = "datomic-transactor-app";
    paths = [
      datomic-pro
      envsubst
      startScript
      propertiesTemplate
    ];
  };

  # System libraries layer
  libsLayer = buildEnv {
    name = "datomic-transactor-libs";
    paths = [
      glibc
      stdenv.cc.cc.lib
      (runCommand "ld-linux-symlink" {} ''
        mkdir -p $out/lib64
        ln -s ${glibc}/lib/ld-linux-x86-64.so.2 $out/lib64/ld-linux-x86-64.so.2
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

      Entrypoint = ["/bin/start-transactor"];

      Env = [
        "PATH=/bin"
        "LD_LIBRARY_PATH=${stdenv.cc.cc.lib}/lib"
      ];

      ExposedPorts."4334/tcp" = {};
      User = "1000:1000";
      WorkingDir = "/${appDir}";
    };
  }
