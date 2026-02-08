# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
FROM docker.io/library/clojure:temurin-21-tools-deps-bookworm-slim AS build

WORKDIR /build

# Cache dependencies first (these layers change less often)
COPY deps.edn ./
RUN clojure -P -M:linux-x86_64
RUN clojure -P -T:build

# Copy source and build the uberjar
COPY build.clj ./
COPY resources/ resources/
COPY src/ src/

RUN clojure -T:build uber

# ---------------------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------------------
FROM docker.io/library/eclipse-temurin:21-jre-noble

RUN groupadd --system bits && useradd --system --gid bits bits

WORKDIR /app

COPY --from=build /build/target/bits.jar bits.jar

USER bits

EXPOSE 3000

ENTRYPOINT ["java"]
CMD ["-XX:+UseG1GC", "-XX:MaxGCPauseMillis=100", "-jar", "bits.jar"]
