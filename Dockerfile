# =============================================================================
# Stage 1: Build the uberjar
# =============================================================================
FROM docker.io/library/clojure:temurin-21-tools-deps-bookworm-slim AS build

WORKDIR /build

# Cache dependency resolution as a separate layer
COPY deps.edn ./
RUN clojure -P
RUN clojure -T:build -P

# Copy source and build the uberjar with AOT + direct linking
COPY build.clj ./
COPY src/ src/
COPY resources/ resources/
RUN clojure -T:build uber

# =============================================================================
# Stage 2: Create a custom JRE with jlink
# =============================================================================
FROM docker.io/library/eclipse-temurin:21-jdk-noble AS jre

COPY --from=build /build/target/bits.jar /tmp/bits.jar

# Determine required Java modules and build a minimal JRE.
# jdeps analyses the uberjar bytecode; jlink strips everything else.
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

# =============================================================================
# Stage 3: Generate AppCDS archive for faster startup
# =============================================================================
FROM docker.io/library/debian:bookworm-slim AS appcds

COPY --from=jre /jre /jre
COPY --from=build /build/target/bits.jar /app/bits.jar

ENV PATH="/jre/bin:${PATH}"

# Run the JVM to dump the class list, then generate the shared archive.
RUN java -XX:DumpLoadedClassList=/tmp/classes.lst \
      -jar /app/bits.jar --dry-run 2>/dev/null || true && \
    java -Xshare:dump \
      -XX:SharedClassListFile=/tmp/classes.lst \
      -XX:SharedArchiveFile=/app/bits.jsa \
      -jar /app/bits.jar 2>/dev/null || true

# =============================================================================
# Stage 4: Minimal runtime image
# =============================================================================
FROM docker.io/library/debian:bookworm-slim AS runtime

RUN groupadd --system bits && \
    useradd --system --gid bits --no-create-home bits

COPY --from=jre /jre /jre
COPY --from=build /build/target/bits.jar /app/bits.jar
COPY --from=appcds /app/bits.jsa /app/bits.jsa

ENV PATH="/jre/bin:${PATH}"

USER bits
WORKDIR /app

EXPOSE 3000

ENTRYPOINT ["java", \
  "-XX:SharedArchiveFile=/app/bits.jsa", \
  "-XX:+UseZGC", \
  "-XX:+ZGenerational", \
  "-Xms256m", \
  "-Xmx512m", \
  "-Dclojure.compiler.direct-linking=true", \
  "-jar", "/app/bits.jar"]
