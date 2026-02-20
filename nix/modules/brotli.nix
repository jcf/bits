{
  lib,
  pkgs,
  ...
}: {
  config = lib.mkIf pkgs.stdenv.isLinux {
    env.LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
    packages = [pkgs.stdenv.cc.cc.lib];
  };
}
