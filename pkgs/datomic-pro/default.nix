{
  fetchzip,
  jre,
  lib,
  makeWrapper,
  postgresql_jdbc,
  stdenv,
  ...
}:
stdenv.mkDerivation (finalAttrs: {
  version = "1.0.7187";
  pname = "datomic-pro";

  src = fetchzip {
    url = "https://datomic-pro-downloads.s3.amazonaws.com/${finalAttrs.version}/datomic-pro-${finalAttrs.version}.zip";
    hash = "sha256-QT+6fozoNQr0Ohic5W+zS2dWEIyso5Sa/OMnwDW1bCA=";
  };

  nativeBuildInputs = [makeWrapper];
  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out/{bin,lib,share}
    cp *transactor*.jar $out/lib/
    mkdir -p $out/share/datomic-pro
    cp -R * $out/share/datomic-pro/

    # Quiet logging config
    cat > $out/share/datomic-pro/resources/logback.xml << 'EOF'
    <configuration>
      <statusListener class="ch.qos.logback.core.status.NopStatusListener" />
      <appender name="CONSOLE" class="ch.qos.logback.core.ConsoleAppender">
        <encoder>
          <charset>UTF-8</charset>
          <pattern>%d{"HH:mm:ss.SSS"} %blue(%-5level) %yellow(%logger{36}) %msg%n</pattern>
        </encoder>
      </appender>
      <root level="WARN">
        <appender-ref ref="CONSOLE" />
      </root>
      <logger name="datomic" level="WARN" />
    </configuration>
    EOF

    makeWrapper ${jre}/bin/java $out/bin/datomic-transactor \
      --set-default "JAVA_OPTS" "-XX:+UseG1GC -XX:MaxGCPauseMillis=50" \
      --add-flags "-server" \
      --add-flags "-cp $out/share/datomic-pro/resources:$out/lib/datomic-transactor-pro-${finalAttrs.version}.jar:$out/share/datomic-pro/lib/*:${postgresql_jdbc}/share/java/postgresql.jar" \
      --add-flags "clojure.main --main datomic.launcher"

    runHook postInstall
  '';

  meta = with lib; {
    homepage = "https://www.datomic.com/";
    description = "Datomic Pro transactional database";
    license = licenses.asl20;
    platforms = platforms.all;
  };
})
