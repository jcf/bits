{
  lib,
  nix2container,
  otel-agent,
  otel-agent-properties,
  pkgs,
  uberjar,
}: let
  inherit (pkgs) binutils buildEnv glibc runCommand stdenv temurin-bin-21 writeTextDir;

  # ---------------------------------------------------------------------------
  # Configuration

  # Use JDK 21 until babashka/fs is fixed for JDK 25
  jdk = temurin-bin-21;
  jdkVersion = "21";

  # Additional JPMS modules beyond what jdeps detects from the uberjar.
  # jdeps finds direct dependencies; these are for reflection, JNDI, etc.
  jreModules = [
    "java.desktop" # java.beans.Introspector (used by HikariCP)
    "java.instrument" # OTEL javaagent instrumentation
    "java.management" # JMX, ManagementFactory (used by Datomic logging)
    "java.naming" # JNDI (used by Datomic, HikariCP)
    "java.sql" # JDBC
    "jdk.crypto.ec" # TLS with EC ciphers
    "jdk.unsupported" # sun.misc.Unsafe (used by Netty, etc.)
  ];

  # JVM runtime options
  jvmOpts = [
    # Garbage collection
    "-XX:+UseZGC"
    "-XX:+ZGenerational"

    # Memory
    "-Xms256m"
    "-Xmx512m"

    # Clojure
    "-Dclojure.compiler.direct-linking=true"
  ];

  # Container paths
  appDir = "app";
  jreDir = "jre";
  tmpDir = "tmp";

  # Native library paths
  libstdcxxPath = "${stdenv.cc.cc.lib}/lib";

  # ---------------------------------------------------------------------------
  # Derived values

  jreModulesStr = lib.concatStringsSep "," jreModules;

  # ---------------------------------------------------------------------------
  # Build components

  # Minimal JRE via jlink with only required modules
  customJre =
    runCommand "bits-jre" {
      nativeBuildInputs = [binutils jdk];
    } ''
      # jdeps needs .jar extension
      cp ${uberjar} $TMPDIR/bits.jar

      # Analyze uberjar for required modules
      detected=$(jdeps \
        --ignore-missing-deps \
        --print-module-deps \
        --multi-release ${jdkVersion} \
        $TMPDIR/bits.jar)

      # Build minimal JRE
      jlink \
        --add-modules "$detected,${jreModulesStr}" \
        --strip-debug \
        --no-man-pages \
        --no-header-files \
        --compress zip-6 \
        --output $out
    '';

  # AppCDS archive for faster startup (~20-30% improvement)
  appCds =
    runCommand "bits-appcds" {
      nativeBuildInputs = [jdk];
      # brotli4j native library needs libstdc++
      LD_LIBRARY_PATH = "${stdenv.cc.cc.lib}/lib";
    } ''
      mkdir -p $out

      # Temp directory for native library extraction (brotli4j)
      export TMPDIR=$PWD/tmp
      mkdir -p $TMPDIR

      # Run warmup to capture loaded classes
      java \
        -Djava.io.tmpdir=$TMPDIR \
        -XX:DumpLoadedClassList=$TMPDIR/classes.lst \
        -jar ${uberjar} --warmup

      # Generate shared archive
      java \
        -Xshare:dump \
        -XX:SharedClassListFile=$TMPDIR/classes.lst \
        -XX:SharedArchiveFile=$out/bits.jsa

      test -f $out/bits.jsa
    '';

  # Application files layer
  appLayer = buildEnv {
    name = "bits-app";
    paths = [
      (runCommand "bits-app-files" {} ''
        mkdir -p $out/${appDir} $out/${tmpDir}
        chmod 1777 $out/${tmpDir}
        cp ${uberjar} $out/${appDir}/bits.jar
        cp ${appCds}/bits.jsa $out/${appDir}/bits.jsa
        cp ${otel-agent}/lib/opentelemetry-javaagent.jar $out/${appDir}/
        cp ${otel-agent-properties} $out/${appDir}/otel-agent.properties
      '')
    ];
  };

  # JRE layer
  jreLayer = buildEnv {
    name = "bits-jre";
    paths = [
      (runCommand "bits-jre-files" {} ''
        mkdir -p $out/${jreDir}
        cp -r ${customJre}/* $out/${jreDir}/
      '')
    ];
  };

  # System libraries layer
  libsLayer = buildEnv {
    name = "bits-libs";
    paths = [
      glibc
      stdenv.cc.cc.lib
      (runCommand "ld-linux-symlink" {} ''
        mkdir -p $out/lib64
        ln -s ${glibc}/lib/ld-linux-x86-64.so.2 $out/lib64/ld-linux-x86-64.so.2
      '')
    ];
  };
  # ---------------------------------------------------------------------------
  # Container
in
  nix2container.buildImage {
    name = "bits";

    copyToRoot = [libsLayer jreLayer appLayer];

    config = {
      Labels = {
        "org.opencontainers.image.description" = "Bits application server";
        "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
        "org.opencontainers.image.title" = "bits";
      };

      Entrypoint =
        [
          "/${jreDir}/bin/java"
          # OTEL agent
          "-javaagent:/${appDir}/opentelemetry-javaagent.jar"
          "-Dotel.javaagent.configuration-file=/${appDir}/otel-agent.properties"
          # AppCDS
          "-XX:SharedArchiveFile=/${appDir}/bits.jsa"
        ]
        ++ jvmOpts
        ++ [
          "-jar"
          "/${appDir}/bits.jar"
        ];

      Env = [
        "LD_LIBRARY_PATH=${libstdcxxPath}"
      ];
      ExposedPorts."3000/tcp" = {};
      User = "1000:1000";
      WorkingDir = "/${appDir}";
    };
  }
