{
  lib,
  stdenv,
  fetchurl,
  autoPatchelfHook,
}: let
  version = "0.1.7";

  sources = {
    "x86_64-linux" = {
      url = "https://github.com/JKamsker/forgejo-cli-ex/releases/download/v${version}/fj-ex-linux-x86_64.tar.gz";
      hash = "sha256-iZkEQlf5384cWtbVePLCOaQeyTmBQGSEEvJIYN+E8S0=";
    };
    "aarch64-darwin" = {
      url = "https://github.com/JKamsker/forgejo-cli-ex/releases/download/v${version}/fj-ex-macos-aarch64.tar.gz";
      hash = "sha256-zMgkpWLxZaZBRkupFEKAkqHqFW5uzdE8iX6vLEw518k=";
    };
  };

  src = fetchurl sources.${stdenv.hostPlatform.system} or (throw "Unsupported platform: ${stdenv.hostPlatform.system}");
in
  stdenv.mkDerivation {
    pname = "forgejo-cli-ex";
    inherit version src;

    nativeBuildInputs = lib.optionals stdenv.isLinux [autoPatchelfHook];

    sourceRoot = ".";

    installPhase = ''
      runHook preInstall
      install -Dm755 fj-ex $out/bin/fj-ex
      runHook postInstall
    '';

    meta = with lib; {
      description = "Extended Forgejo CLI with action logs, artifacts, and more";
      homepage = "https://github.com/JKamsker/forgejo-cli-ex";
      license = licenses.mit;
      platforms = ["x86_64-linux" "aarch64-darwin"];
      mainProgram = "fj-ex";
    };
  }
