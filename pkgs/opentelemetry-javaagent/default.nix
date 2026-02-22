{
  fetchurl,
  lib,
  stdenv,
}: let
  version = "2.14.0";
in
  stdenv.mkDerivation {
    inherit version;
    pname = "opentelemetry-javaagent";

    src = fetchurl {
      url = "https://github.com/open-telemetry/opentelemetry-java-instrumentation/releases/download/v${version}/opentelemetry-javaagent.jar";
      hash = "sha256-Fvjij6HdzVbthb9jO9HR+8eOp8TMUOjFcmsqMZ9QWMg=";
    };

    dontUnpack = true;
    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      runHook preInstall
      mkdir -p $out/lib
      cp $src $out/lib/opentelemetry-javaagent.jar
      runHook postInstall
    '';

    meta = with lib; {
      homepage = "https://opentelemetry.io/docs/zero-code/java/agent/";
      description = "OpenTelemetry auto-instrumentation agent for Java";
      license = licenses.asl20;
    };
  }
