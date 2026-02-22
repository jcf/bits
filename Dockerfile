# Stage 1: Build the uberjar
FROM docker.io/library/clojure:temurin-21-tools-deps-bookworm-slim AS build

WORKDIR /build

COPY deps.edn build.clj ./
RUN clojure -P
RUN clojure -P -A:build
COPY src/ src/
COPY resources/ resources/
RUN clojure -T:build uber

# Stage 2: Download OpenTelemetry Java Agent
FROM docker.io/library/debian:bookworm-slim AS otel
ARG OTEL_AGENT_VERSION=2.14.0
ADD https://github.com/open-telemetry/opentelemetry-java-instrumentation/releases/download/v${OTEL_AGENT_VERSION}/opentelemetry-javaagent.jar \
    /otel/opentelemetry-javaagent.jar

# Stage 3: Create a custom JRE with jlink
FROM docker.io/library/eclipse-temurin:21-jdk-noble AS jre

COPY --from=build /build/target/bits.jar /tmp/bits.jar

# jdeps analyses bytecode; jlink builds a minimal JRE with only required modules
RUN jdeps \
      --ignore-missing-deps \
      --print-module-deps \
      --multi-release 21 \
      /tmp/bits.jar > /tmp/modules.txt && \
    jlink \
      --add-modules "$(cat /tmp/modules.txt),jdk.crypto.ec,jdk.unsupported" \
      --strip-debug \
      --no-man-pages \
      --no-header-files \
      --compress zip-6 \
      --output /jre

# Stage 4: Generate AppCDS archive for faster startup
FROM docker.io/library/debian:bookworm-slim AS appcds

COPY --from=jre /jre /jre
COPY --from=build /build/target/bits.jar /app/bits.jar

ENV PATH="/jre/bin:${PATH}"

# --warmup loads classes then exits; we dump them for ~20-30% faster cold starts
RUN java -XX:DumpLoadedClassList=/tmp/classes.lst \
      -jar /app/bits.jar --warmup 2>/dev/null || true && \
    java -Xshare:dump \
      -XX:SharedClassListFile=/tmp/classes.lst \
      -XX:SharedArchiveFile=/app/bits.jsa \
      -jar /app/bits.jar 2>/dev/null || true

# Stage 5: Minimal runtime image
FROM docker.io/library/debian:bookworm-slim AS runtime

RUN groupadd --system bits && \
    useradd --system --gid bits --no-create-home bits

COPY --from=jre /jre /jre
COPY --from=build /build/target/bits.jar /app/bits.jar
COPY --from=appcds /app/bits.jsa /app/bits.jsa
COPY --from=otel /otel/opentelemetry-javaagent.jar /app/
COPY --from=build /build/resources/otel-agent.properties /app/

ENV PATH="/jre/bin:${PATH}"

USER bits
WORKDIR /app

EXPOSE 3000

ENTRYPOINT ["java", \
  "-javaagent:/app/opentelemetry-javaagent.jar", \
  "-Dotel.javaagent.configuration-file=/app/otel-agent.properties", \
  "-XX:SharedArchiveFile=/app/bits.jsa", \
  "-XX:+UseZGC", \
  "-XX:+ZGenerational", \
  "-Xms256m", \
  "-Xmx512m", \
  "-Dclojure.compiler.direct-linking=true", \
  "-jar", "/app/bits.jar"]
