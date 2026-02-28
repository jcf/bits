{
  cacert,
  clojure,
  esbuild,
  fake-git,
  jdk,
  lib,
  lockfile,
  mk-deps-cache,
  stdenv,
  tailwindcss_4,
  version ? "dev",
}: let
  root = ../..;

  # Build aliases (and architectures) to include in the uberjar build
  buildAliases = [
    "linux-aarch64"
    "linux-x86_64"
  ];

  # Build aliases as Clojure vector: [:alias1 :alias2]
  buildAliasesEdn = "[${lib.concatMapStringsSep " " (a: ":${a}") buildAliases}]";

  clj = clojure.override {jdk = jdk;};
  sslCertFile = "${cacert}/etc/ssl/certs/ca-bundle.crt";

  # Source files (deps-lock.json provided via flake input, not in fileset)
  fullSrc = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      (root + "/build.clj")
      (root + "/deps.edn")
      (root + "/resources")
      (root + "/src")
    ];
  };

  # clj-nix deps cache from lock file
  depsCache = mk-deps-cache {
    inherit lockfile;
  };
in
  stdenv.mkDerivation {
    pname = "bits-uberjar";
    inherit version;
    src = fullSrc;
    nativeBuildInputs = [cacert clj esbuild fake-git jdk tailwindcss_4];

    SSL_CERT_FILE = sslCertFile;
    GIT_SSL_CAINFO = sslCertFile;

    # brotli4j extracts native libs that need libstdc++
    LD_LIBRARY_PATH = "${stdenv.cc.cc.lib}/lib";

    # Clojure environment variables
    # https://clojure.org/reference/deps_and_cli#_clojure_cli_usage
    buildPhase = ''
      runHook preBuild

      # Generate Tailwind CSS
      mkdir -p resources/public
      tailwindcss --input resources/tailwind.css --output resources/public/app.css

      # Minify JavaScript
      esbuild resources/public/bits.js \
        --minify \
        --outfile=resources/public/bits.js \
        --allow-overwrite

      # Point HOME directly to deps cache (contains .m2, .gitlibs, .clojure)
      export HOME="${depsCache}"
      export JAVA_TOOL_OPTIONS="-Duser.home=${depsCache}"

      # Clojure CLI environment
      export CLJ_CONFIG="$HOME/.clojure"
      export CLJ_CACHE="$TMP/cp_cache"
      export GITLIBS="$HOME/.gitlibs"

      clojure -T:build uber :aliases '${buildAliasesEdn}'

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      cp target/bits.jar $out
      runHook postInstall
    '';

    passthru = {inherit depsCache;};
  }
