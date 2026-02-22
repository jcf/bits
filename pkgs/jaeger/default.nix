{
  fetchzip,
  lib,
  stdenv,
}:
let
  version = "2.15.1";

  platforms = {
    x86_64-linux = {
      name = "linux-amd64";
      hash = "sha256-kAOXKD8aIDt5pIzflpc9Ccdl8WxMfPkgbrM6ySg42iY=";
    };
    aarch64-linux = {
      name = "linux-arm64";
      hash = "sha256-RYcdTwoXnFT8bWjba/qwsgb0MIM+dJ1KzoxI6OOjhf0=";
    };
    x86_64-darwin = {
      name = "darwin-amd64";
      hash = "sha256-BX8v1Dqc/Go2LRSHeLtmN7DUVZz33OIZCm+MwBlgnOw=";
    };
    aarch64-darwin = {
      name = "darwin-arm64";
      hash = "sha256-ursyy//pzjFSD9p3pTi1EqXYmStbizwjJPxqLOjwOvk=";
    };
  };

  platform = platforms.${stdenv.hostPlatform.system}
    or (throw "Unsupported platform: ${stdenv.hostPlatform.system}");
in
stdenv.mkDerivation {
  inherit version;
  pname = "jaeger";

  src = fetchzip {
    url = "https://github.com/jaegertracing/jaeger/releases/download/v${version}/jaeger-${version}-${platform.name}.tar.gz";
    hash = platform.hash;
  };

  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    cp jaeger $out/bin/
    runHook postInstall
  '';

  meta = with lib; {
    homepage = "https://www.jaegertracing.io/";
    description = "Jaeger distributed tracing platform";
    license = licenses.asl20;
    platforms = builtins.attrNames platforms;
  };
}
