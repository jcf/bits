{
  container-base,
  init-sql,
  nix2container,
  pkgs,
}: let
  inherit (pkgs) buildEnv busybox runCommand writeTextDir;

  # Postgres needs root for the initdb privilege-drop pattern
  passwd = writeTextDir "etc/passwd" "root:x:0:0:root:/root:/bin/sh\npostgres:x:${container-base.uid}:${container-base.uid}:postgres:/var/lib/postgresql:/bin/sh\n";
  group = writeTextDir "etc/group" "root:x:0:\npostgres:x:${container-base.uid}:\n";

  postgresql = pkgs.postgresql_17.withPackages (ps: [
    ps.pgvector
    ps.postgis
  ]);

  dataDir = "var/lib/postgresql/data";

  # docker-library/postgres pattern: start as root → chown → re-exec as postgres
  startupScript = writeTextDir "app/bin/start-postgres" ''
    #!/bin/sh
    set -eu

    PGDATA=/${dataDir}

    if [ "$(id -u)" = '0' ]; then
      mkdir -p "$PGDATA"
      chown -R ${container-base.uid}:${container-base.uid} /var/lib/postgresql
      exec su -s /bin/sh postgres -c "exec /bin/sh /app/bin/start-postgres"
    fi

    if [ ! -f "$PGDATA/PG_VERSION" ]; then
      /app/bin/initdb \
        --username=postgres \
        --encoding=UTF-8 \
        --locale=C \
        --auth=trust \
        -D "$PGDATA"

      echo "host all all 0.0.0.0/0 md5" >> "$PGDATA/pg_hba.conf"
      echo "host all all ::/0 md5" >> "$PGDATA/pg_hba.conf"
      echo "local all all trust" >> "$PGDATA/pg_hba.conf"
      echo "listen_addresses = '0.0.0.0'" >> "$PGDATA/postgresql.conf"
      echo "unix_socket_directories = '/${dataDir}'" >> "$PGDATA/postgresql.conf"

      /app/bin/pg_ctl -D "$PGDATA" -w start
      /app/bin/psql -h /${dataDir} -U postgres -f /app/init.sql
      /app/bin/pg_ctl -D "$PGDATA" -w stop
    fi

    exec /app/bin/postgres -D "$PGDATA"
  '';

  files = runCommand "postgres-files" {} ''
    mkdir -p $out/app/bin $out/app/lib $out/app/share $out/bin

    for bin in initdb pg_ctl pg_isready postgres psql; do
      cp ${postgresql}/bin/$bin $out/app/bin/
    done

    cp -r ${postgresql}/lib/* $out/app/lib/
    cp -r ${postgresql}/share/* $out/app/share/
    cp ${init-sql} $out/app/init.sql

    for cmd in chown id mkdir sh su; do
      ln -s ${busybox}/bin/$cmd $out/bin/$cmd
    done
  '';

  rootLayer = buildEnv {
    name = "postgres-root";
    paths = [
      container-base.etcNsswitch
      container-base.syslibs
      files
      group
      passwd
      startupScript
    ];
  };
in
  nix2container.buildImage {
    name = "bits-postgres";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-postgres" "PostgreSQL 17 with pgvector and PostGIS";

      Entrypoint = ["/bin/sh" "/app/bin/start-postgres"];

      Env = [
        "LD_LIBRARY_PATH=/app/lib:/lib"
        "PATH=/app/bin:/bin"
        "PGDATA=/${dataDir}"
      ];

      ExposedPorts."5432/tcp" = {};
      WorkingDir = "/app";

      Volumes."/${dataDir}" = {};
    };
  }
