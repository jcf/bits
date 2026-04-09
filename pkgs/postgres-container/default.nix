{
  nix2container,
  pkgs,
  init-sql,
}: let
  inherit (pkgs) buildEnv busybox glibc runCommand stdenv writeTextDir;

  passwd = writeTextDir "etc/passwd" "root:x:0:0:root:/root:/bin/sh\npostgres:x:1000:1000:postgres:/var/lib/postgresql:/bin/sh\n";
  group = writeTextDir "etc/group" "root:x:0:\npostgres:x:1000:\n";
  nsswitch = writeTextDir "etc/nsswitch.conf" "passwd: files\ngroup: files\n";

  postgresql = pkgs.postgresql_17.withPackages (ps: [
    ps.pgvector
    ps.postgis
  ]);

  appDir = "app";
  dataDir = "var/lib/postgresql/data";

  # Entrypoint follows the docker-library/postgres pattern:
  # start as root → fix ownership → re-exec as postgres via gosu/su.
  startupScript = writeTextDir "${appDir}/bin/start-postgres" ''
    #!/bin/sh
    set -eu

    PGDATA=/${dataDir}

    if [ "$(id -u)" = '0' ]; then
      mkdir -p "$PGDATA"
      chown -R 1000:1000 /var/lib/postgresql
      exec su -s /bin/sh postgres -c "exec /bin/sh /${appDir}/bin/start-postgres"
    fi

    if [ ! -f "$PGDATA/PG_VERSION" ]; then
      /${appDir}/bin/initdb \
        --username=postgres \
        --encoding=UTF-8 \
        --locale=C \
        --auth=trust \
        -D "$PGDATA"

      echo "host all all 0.0.0.0/0 md5" >> "$PGDATA/pg_hba.conf"
      echo "host all all ::/0 md5" >> "$PGDATA/pg_hba.conf"
      echo "local all all trust" >> "$PGDATA/pg_hba.conf"
      echo "listen_addresses = '0.0.0.0'" >> "$PGDATA/postgresql.conf"
      echo "unix_socket_directories = '/$PGDATA'" >> "$PGDATA/postgresql.conf"

      /${appDir}/bin/pg_ctl -D "$PGDATA" -w start
      /${appDir}/bin/psql -h /${dataDir} -U postgres -f /${appDir}/init.sql
      /${appDir}/bin/pg_ctl -D "$PGDATA" -w stop
    fi

    exec /${appDir}/bin/postgres -D "$PGDATA"
  '';

  libs = runCommand "postgres-libs" {} ''
    mkdir -p $out/lib
    cp -r ${glibc}/lib/* $out/lib/
    cp -r ${stdenv.cc.cc.lib}/lib/* $out/lib/

    ${
      if stdenv.hostPlatform.isAarch64
      then ""
      else ''
        mkdir -p $out/lib64
        ln -s /lib/ld-linux-x86-64.so.2 $out/lib64/ld-linux-x86-64.so.2
      ''
    }
  '';

  files = runCommand "postgres-files" {} ''
    mkdir -p $out/${appDir}/bin $out/${appDir}/lib $out/${appDir}/share $out/tmp

    for bin in initdb pg_ctl pg_isready postgres psql; do
      cp ${postgresql}/bin/$bin $out/${appDir}/bin/
    done

    cp -r ${postgresql}/lib/* $out/${appDir}/lib/
    cp -r ${postgresql}/share/* $out/${appDir}/share/
    cp ${init-sql} $out/${appDir}/init.sql

    mkdir -p $out/bin
    for cmd in chown id mkdir sh su; do
      ln -s ${busybox}/bin/$cmd $out/bin/$cmd
    done
  '';

  rootLayer = buildEnv {
    name = "postgres-root";
    paths = [files group libs nsswitch passwd startupScript];
  };
in
  nix2container.buildImage {
    name = "bits-postgres";

    copyToRoot = [rootLayer];

    config = {
      Labels = {
        "org.opencontainers.image.description" = "PostgreSQL 17 with pgvector and PostGIS";
        "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
        "org.opencontainers.image.title" = "bits-postgres";
      };

      Entrypoint = ["/bin/sh" "/${appDir}/bin/start-postgres"];

      Env = [
        "LD_LIBRARY_PATH=/${appDir}/lib:/lib"
        "PATH=/${appDir}/bin:/bin"
        "PGDATA=/${dataDir}"
      ];

      ExposedPorts."5432/tcp" = {};
      WorkingDir = "/${appDir}";
    };
  }
