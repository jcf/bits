{
  lib,
  nix2container,
  pkgs,
}: let
  # Shared CI packages (keep in sync with devenv.nix)
  ci = import ../../nix/ci.nix {inherit pkgs;};

  inherit
    (pkgs)
    attic-client
    bash
    buildEnv
    cacert
    coreutils
    curl
    devenv
    firefox
    geckodriver
    git
    gnugrep
    gnutar
    gzip
    jq
    ncurses
    nix
    nodejs
    openssh
    podman
    postgresql
    runCommand
    skopeo
    stdenv
    writeTextDir
    zsh
    ;

  # Packages available in the CI container
  packages =
    [
      attic-client
      bash
      cacert
      coreutils
      curl
      devenv
      firefox
      geckodriver
      git
      gnugrep
      gnutar
      gzip
      jq
      ncurses
      nix
      nodejs
      openssh
      podman
      postgresql
      skopeo
      stdenv.cc.cc.lib
      zsh
    ]
    ++ ci.packages;

  # Nix configuration for single-user mode
  nixConf = writeTextDir "etc/nix/nix.conf" ''
    experimental-features = nix-command flakes
    sandbox = false
    accept-flake-config = true
    build-users-group =
  '';

  # Basic passwd/group for container
  passwd = writeTextDir "etc/passwd" ''
    root:x:0:0:root:/root:/bin/bash
  '';

  group = writeTextDir "etc/group" ''
    root:x:0:
  '';

  # NSS config for user lookups
  nsswitch = writeTextDir "etc/nsswitch.conf" ''
    passwd: files
    group: files
    shadow: files
    hosts: files dns
  '';

  # Container policy for podman/skopeo
  containersPolicy = writeTextDir "etc/containers/policy.json" ''
    {"default": [{"type": "insecureAcceptAnything"}]}
  '';

  # /usr/bin/env for scripts with #!/usr/bin/env shebang
  usrBinEnv = runCommand "usr-bin-env" {} ''
    mkdir -p $out/usr/bin
    ln -s ${coreutils}/bin/env $out/usr/bin/env
  '';

  # Directories layer
  dirsLayer = runCommand "ci-dirs" {} ''
    mkdir -p $out/root $out/tmp $out/nix/var/nix
    chmod 1777 $out/tmp
  '';

  # Config files layer
  configLayer = buildEnv {
    name = "ci-config";
    paths = [containersPolicy nixConf nsswitch passwd group usrBinEnv];
  };

  # Packages layer
  packagesLayer = buildEnv {
    name = "ci-packages";
    paths = packages;
  };
in
  nix2container.buildImage {
    name = "bits-ci";

    copyToRoot = [dirsLayer configLayer packagesLayer];

    config = {
      Env = [
        "HOME=/root"
        "LD_LIBRARY_PATH=${stdenv.cc.cc.lib}/lib"
        "NIX_PATH=nixpkgs=${pkgs.path}"
        "PATH=/bin:/usr/bin:${lib.makeBinPath packages}"
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
        "USER=root"
      ];
      WorkingDir = "/root";
      Cmd = ["/bin/bash"];
    };
  }
