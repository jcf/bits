{
  container-base,
  lib,
  nix2container,
  otel-agent,
  pkgs,
}: let
  inherit (pkgs) buildEnv busybox cacert coreutils git runCommand stdenv temurin-bin-21 writeTextDir;

  jdk = temurin-bin-21;
  clj = pkgs.clojure.override {jdk = jdk;};
  user = container-base.user;
  uid = container-base.uid;

  passwd = writeTextDir "etc/passwd" "root:x:0:0:root:/root:/bin/sh\n${user}:x:${uid}:${uid}:${user}:/home/${user}:/bin/sh\n";
  group = writeTextDir "etc/group" "root:x:0:\n${user}:x:${uid}:\n";

  usrBinEnv = runCommand "usr-bin-env" {} ''
    mkdir -p $out/usr/bin
    ln -s ${coreutils}/bin/env $out/usr/bin/env
  '';

  entrypoint = pkgs.writeTextDir "entrypoint.d/run" ''
    #!/bin/sh
    set -eu
    if [ "$(id -u)" = '0' ]; then
      mkdir -p /home/${user}
      chown -R ${uid}:${uid} /home/${user}
      exec su -s /bin/sh ${user} -c "exec /bin/sh /entrypoint.d/run $*"
    fi
    exec "$@"
  '';

  dirs = runCommand "dev-dirs" {} ''
    mkdir -p $out/app/bin $out/bin $out/tmp
    chmod 1777 $out/tmp

    for cmd in chown id mkdir sh su; do
      ln -s ${busybox}/bin/$cmd $out/bin/$cmd
    done
  '';

  rootLayer = buildEnv {
    name = "dev-root";
    paths = [
      cacert
      clj
      container-base.etcNsswitch
      container-base.syslibs
      coreutils
      dirs
      entrypoint
      git
      group
      jdk
      otel-agent
      passwd
      usrBinEnv
    ];
  };
in
  nix2container.buildImage {
    name = "bits-dev";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-dev" "Bits development environment";

      Entrypoint = ["/bin/sh" "/entrypoint.d/run"];

      Env = [
        "HOME=/home/${user}"
        "LD_LIBRARY_PATH=${stdenv.cc.cc.lib}/lib"
        "OTEL_JAVAAGENT_PATH=${otel-agent}/lib/opentelemetry-javaagent.jar"
        "PATH=/bin:/usr/bin:${lib.makeBinPath [coreutils clj git jdk]}"
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
        "USER=${user}"
      ];

      ExposedPorts = {
        "3000/tcp" = {};
        "9999/tcp" = {};
      };

      WorkingDir = "/app";
    };
  }
