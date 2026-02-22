# Terraform will panic if it encounters `PGSERVICEFILE`.

unexport PGSERVICEFILE

os := "darwin-aarch64"
plan_dir := justfile_directory() / ".terraform-plans"

_default:
    @just fmt
    @just lint
    @just test
    @just compile

# ------------------------------------------------------------------------------
# Docs

# Create a new decision record
[group('docs')]
decide +title:
    #!/usr/bin/env bash
    timestamp=$(date +%Y%m%d%H%M%S)
    normalized=$(echo "{{ title }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    author="$(git config user.name) <$(git config user.email)>"
    filename="decisions/${timestamp}-${normalized}.org"
    cat > "$filename" <<EOF
    #+title:  {{ title }}
    #+author: ${author}
    #+date:   $(date +%Y-%m-%d)
    EOF
    echo "🎯 {{ BOLD }}Created \"$filename\"{{ NORMAL }}."

# ------------------------------------------------------------------------------
# Development

# Create self-signed SSL certificates via mkcert
[group('dev')]
mkcert:
    #!/usr/bin/env zsh
    set -e

    mkcert -install
    mkdir -p {{ justfile_directory() }}/certs
    cd {{ justfile_directory() }}/certs

    # We append .test to all domains during development. The following domains
    # need to be supported:
    #
    # -- usebits.app
    # api.usebits.app
    # edit.usebits.app
    # www.usebits.app
    #
    # -- bits.page
    # bits.page
    # jcf.bits.page
    domains=(
        usebits.app.test
        app.test

        bits.page.test
        page.test

        test
    )

    for domain in $domains; do
        if [[ ! -f "_wildcard.${domain}.pem" ]]; then
            mkcert "*.${domain}"
        fi
    done

    echo >&2 "🔒 Wildcard certificates ready!"

# Generate cluster keystores
[group('dev')]
cluster-certs:
    #!/usr/bin/env zsh
    set -e
    mkdir -p certs

    if [[ ! -f certs/cluster-keystore.p12 ]]; then
        keytool -genkeypair \
            -alias bits-cluster \
            -keyalg EC \
            -groupname secp256r1 \
            -validity 3650 \
            -storetype PKCS12 \
            -keystore certs/cluster-keystore.p12 \
            -storepass "$CLUSTER_KEYSTORE_PASSWORD" \
            -dname "CN=bits-cluster,O=Invetica" \
            -ext "SAN=IP:127.0.0.1"

        keytool -genkeypair \
            -alias rogue \
            -keyalg EC \
            -groupname secp256r1 \
            -validity 1 \
            -storetype PKCS12 \
            -keystore certs/rogue-keystore.p12 \
            -storepass password \
            -dname "CN=rogue,O=Evil"
    fi

    echo >&2 "🔒 Cluster certificates ready!"

# Setup a local development environment
[group('dev')]
setup:
    @just mkcert
    @just cluster-certs
    devenv shell true
    pnpm install
    @echo -e "\n✅ {{ BOLD }}Setup complete!{{ NORMAL }}"

# Format project files
[group('dev')]
fmt:
    treefmt

# Clean up generated classes and screenshots
[group('dev')]
clean:
    @rm -r target/classes
    @rm -r target/screenshots

[group('dev')]
nrepl *args:
    #!/usr/bin/env zsh
    set -e

    echo >&2 \
        "{{ BLUE }}{{ BOLD }}==>{{ NORMAL }} {{ BOLD }}Starting nREPL on localhost:9999...{{ NORMAL }}"

    exec clojure \
        -M:dev:test:logging:otel:nrepl:{{ os }} \
        --report stderr \
        --main bits.nrepl \
        --host localhost \
        --port 9999 \
        {{ args }}

# Watch source code for Tailwind classes
[group('dev')]
tailwind:
    mkdir -p resources/public
    pnpm --filter @bits/tailwind \
        exec tailwindcss \
            --watch \
            --input ../../resources/tailwind.css \
            --output ../../resources/public/app.css

# Run the marketing site
[group('dev')]
market:
    pnpm --filter @bits/www dev

# Regenerate Tailwind CSS from template
[group('dev')]
css:
    clojure -M:dev -m bits.dev.assets

# Import clj-kondo configs
[group('dev')]
clj-kondo-import:
    clj-kondo --lint "$(clojure -Spath)" --dependencies --skip-lint --copy-configs

# ------------------------------------------------------------------------------
# Locales

# Extract translatable strings to .pot file
[group('locales')]
locales-extract:
    clojure -T:build locales-extract

# Build translation bundles from .po files
[group('locales')]
locales-build:
    clojure -T:build build-translations

# ------------------------------------------------------------------------------
# Test

# Run lints
[group('test')]
lint:
    clj-kondo --lint dev src test

# Compile bits.main
[group('test')]
compile:
    clojure -M:dev --report stderr -m bits.dev.compile

# Run tests
[group('test')]
test *args:
    clojure -M:dev:test:runner:{{ os }} {{ args }}

# Run tests with performance tracing output
[group('test')]
perf *args:
    OTEL_TRACES_EXPORTER=logging-otlp clojure -M:dev:test:otel:runner:{{ os }} {{ args }}

# ------------------------------------------------------------------------------
# Build

# Build an AOT-compiled uberjar
[group('build')]
build:
    clojure -T:build uber

# Build Datomic Pro output
[group('build')]
build-datomic:
    devenv build outputs.datomic-pro

# Build the Docker image
[group('build')]
docker-build tag="bits:latest":
    docker build -t {{ tag }} .

# Run the Docker image against the local devenv database
[group('build')]
docker-run tag="bits:latest" *args:
    docker run --rm -p 3000:3000 \
        -e DATABASE_URL=jdbc:postgresql://host.docker.internal:5432/bits_dev?user=bits\&password=please \
        -e CSRF_SECRET=default-csrf-secret-change-in-prod \
        --add-host=host.docker.internal:host-gateway \
        {{ args }} {{ tag }}

# ------------------------------------------------------------------------------
# PostgreSQL

# Start an interactive psql session connected to the local development database
[group('postgres')]
psql *args:
    PGPASSWD=please psql \
        --host=localhost \
        --port=5432 \
        --username=bits \
        --dbname=bits_dev \
        {{ args }}

# Create a new migration
[group('postgres')]
migration name:
    #!/usr/bin/env zsh
    set -e
    timestamp="$(date +%Y%m%d%H%M%S)"
    normalized=$(echo "{{ name }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    dir="resources/migrations"
    mkdir -p "$dir"
    for direction in up down; do
        path="$dir/${timestamp}-${normalized}.${direction}.sql"
        touch "$path" && echo "$path"
    done

# ------------------------------------------------------------------------------
# Infrastructure

[group('iac')]
_terraform dir *args:
    op run -- terraform -chdir={{ justfile_directory() }}/iac/{{ dir }} {{ args }}

# Initialize one or all Terraform projects
[group('iac')]
init dir *args:
    @just _terraform {{ dir }} init {{ args }}

# Plan one or all Terraform projects
[group('iac')]
plan dir:
    @mkdir -p {{ plan_dir }}
    @just _terraform {{ dir }} plan -out {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan

# Apply one or all Terraform projects
[group('iac')]
apply dir:
    @just _terraform {{ dir }} apply {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan
    rm {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan

# Interact with outputs one or all Terraform projects
[group('iac')]
output dir *args:
    @just _terraform {{ dir }} output {{ args }}
