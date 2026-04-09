# Shared foundation for all bits container images.
#
# Provides:
#   - System libraries (glibc, libstdc++)
#   - User `bits` (uid 1000) with /etc/passwd, /etc/group, /etc/nsswitch.conf
#   - Writable /tmp (mode 1777)
#   - Helper functions for building consistent container configs
{pkgs}: let
  inherit (pkgs) buildEnv glibc runCommand stdenv writeTextDir;

  user = "bits";
  uid = "1000";

  etcGroup = writeTextDir "etc/group" "${user}:x:${uid}:\n";
  etcNsswitch = writeTextDir "etc/nsswitch.conf" "passwd: files\ngroup: files\nhosts: files dns\n";
  etcPasswd = writeTextDir "etc/passwd" "${user}:x:${uid}:${uid}:${user}:/home/${user}:/bin/sh\n";

  etcFiles = buildEnv {
    name = "container-etc";
    paths = [etcGroup etcNsswitch etcPasswd];
  };

  syslibs = runCommand "container-syslibs" {} ''
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

  dirs = runCommand "container-dirs" {} ''
    mkdir -p $out/tmp
    chmod 1777 $out/tmp
  '';
in {
  inherit dirs etcFiles etcGroup etcNsswitch etcPasswd syslibs uid user;

  # Common paths to include in every container's buildEnv
  paths = [dirs etcFiles syslibs];

  # Standard OCI labels
  labels = name: description: {
    "org.opencontainers.image.description" = description;
    "org.opencontainers.image.source" = "https://code.invetica.team/jcf/bits";
    "org.opencontainers.image.title" = name;
  };
}
