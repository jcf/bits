{
  lib,
  pkgsLinux,
  version ? "dev",
}: let
  # Shared CI packages (keep in sync with devenv.nix)
  ci = import ../../nix/ci.nix {pkgs = pkgsLinux;};

  inherit
    (pkgsLinux)
    attic-client
    bash
    cacert
    coreutils
    curl
    devenv
    dockerTools
    firefox
    geckodriver
    git
    gnugrep
    gnutar
    gzip
    jq
    nix
    nodejs
    openssh
    podman
    postgresql
    skopeo
    stdenv
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
  nixConf = pkgsLinux.writeTextDir "etc/nix/nix.conf" ''
    experimental-features = nix-command flakes
    sandbox = false
    accept-flake-config = true
    build-users-group =
  '';

  # Basic passwd/group for container
  passwd = pkgsLinux.writeTextDir "etc/passwd" ''
    root:x:0:0:root:/root:/bin/bash
  '';

  group = pkgsLinux.writeTextDir "etc/group" ''
    root:x:0:
  '';

  # NSS config for user lookups
  nsswitch = pkgsLinux.writeTextDir "etc/nsswitch.conf" ''
    passwd: files
    group: files
    shadow: files
    hosts: files dns
  '';

  # /usr/bin/env for scripts with #!/usr/bin/env shebang
  usrBinEnv = pkgsLinux.runCommand "usr-bin-env" {} ''
    mkdir -p $out/usr/bin
    ln -s ${coreutils}/bin/env $out/usr/bin/env
  '';
in
  dockerTools.buildLayeredImage {
    name = "git.lan.invetica.co.uk/jcf/bits/bits-ci";
    tag = version;

    contents =
      packages
      ++ [
        nixConf
        passwd
        group
        nsswitch
        usrBinEnv
      ];

    extraCommands = ''
      mkdir -p root tmp nix/var/nix
      chmod 1777 tmp
    '';

    config = {
      Env = [
        "HOME=/root"
        "LD_LIBRARY_PATH=${stdenv.cc.cc.lib}/lib"
        "NIX_PATH=nixpkgs=${pkgsLinux.path}"
        "PATH=/bin:/usr/bin:${lib.makeBinPath packages}"
        "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
        "USER=root"
      ];
      WorkingDir = "/root";
      Cmd = ["/bin/bash"];
    };
  }
