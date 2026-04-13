# Terraform will panic if it encounters `PGSERVICEFILE`.

unexport PGSERVICEFILE

os := "linux-aarch64"
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

    # Wildcard certs for .localhost subdomains (auto-resolve to 127.0.0.1)
    domains=(
        bits.page.localhost
        page.localhost
        localhost
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
    @just build-containers
    devenv shell true
    @echo -e "\n✅ {{ BOLD }}Setup complete!{{ NORMAL }}"

# Build and install container images (skips unchanged)
[group('dev')]
build-containers:
    #!/usr/bin/env zsh
    set -e
    state="{{ justfile_directory() }}/target/container-images"
    mkdir -p "$state"
    typeset -A images=(
        bits-dev              dev-container-arm64
        bits-error-pages      error-pages-container-arm64
        bits-jaeger           jaeger-container-arm64
        bits-postgres         postgres-container-arm64
        bits-tailwind         tailwind-container-arm64
        bits-traefik          traefik-container-arm64
        bits-transactor       transactor-container-arm64
    )
    for tag flake in "${(@kv)images}"; do
        store_path=$(nix build ".#${flake}" --no-link --print-out-paths)
        stamp="$state/${tag}"
        if [[ -f "$stamp" ]] && [[ "$(cat "$stamp")" == "$store_path" ]]; then
            echo >&2 "{{ BLUE }}{{ BOLD }}→{{ NORMAL }} {{ BOLD }}${tag}{{ NORMAL }} unchanged, skipping"
            continue
        fi
        echo >&2 "{{ BLUE }}{{ BOLD }}→{{ NORMAL }} {{ BOLD }}Loading ${tag}...{{ NORMAL }}"
        nix run ".#${flake}.copyTo" -- "docker-daemon:${tag}:latest"
        echo "$store_path" > "$stamp"
    done
    echo >&2 "{{ BOLD }}✅ All images loaded.{{ NORMAL }}"

# Run Docker compose commands
[group('dev')]
docker *args:
    docker compose {{ args }}

# Start the full development stack via Docker Compose
[group('dev')]
dev *args:
    @just docker up {{ args }}

# Run a one-shot command inside a throwaway nrepl container
[group('dev')]
run *args:
    @just docker run --rm --no-TTY nrepl {{ args }}

# Format project files
[group('dev')]
fmt *args:
    treefmt {{ args }}

# Clean up generated classes and screenshots
[group('dev')]
clean:
    @rm -r \
        target/browser-sessions \
        target/classes \
        target/screenshots

# Regenerate Tailwind CSS from template
[group('dev')]
css:
    @just run clojure -M:dev --report stderr -m bits.dev.assets

# Import clj-kondo configs
[group('dev')]
clj-kondo-import:
    @just docker run --rm --no-TTY nrepl sh -c 'clj-kondo --lint "$(clojure -Spath)" --dependencies --skip-lint --copy-configs'

# Run bits.cli with given args
[group('dev')]
cli *args:
    @just run clojure -M:cli --report stderr -m bits.cli {{ args }}

# ------------------------------------------------------------------------------
# Locales

# Extract translatable strings to .pot file
[group('locales')]
locales-extract:
    @just run clojure -T:build --report stderr locales-extract

# Build translation bundles from .po files
[group('locales')]
locales-build:
    @just run clojure -T:build --report stderr build-translations

# ------------------------------------------------------------------------------
# Test

# Run all quality checks (format + lint)
[group('test')]
check:
    treefmt --fail-on-change
    clj-kondo --lint dev src test

# Run lints
[group('test')]
lint:
    clj-kondo --lint dev src test

# Compile bits namespaces
[group('test')]
compile:
    @just run clojure -M:cli -m bits.cli warmup

# Run tests
[group('test')]
test *args:
    @just run clojure -M:dev:test:runner:{{ os }} {{ args }}

# Run tests with spans logged to stdout
[group('test')]
perf *args:
    @just run env OTEL_TRACES_EXPORTER=logging-otlp clojure -M:dev:test:otel:runner:{{ os }} {{ args }}

# ------------------------------------------------------------------------------
# Build

# Regenerate deps-lock.json
[group('build')]
deps-lock:
    nix run 'git+https://code.invetica.team/jcf/clj-nix?rev=5751969234c45823955a0fd348831068f1107453#deps-lock'

# Build an AOT-compiled uberjar
[group('build')]
build:
    nix build .#bits-uberjar

# Build Datomic Pro output
[group('build')]
build-datomic:
    nix build .#datomic-pro

# Build the production container image and load into local Docker
[group('build')]
container:
    nix run .#bits-container-arm64.copyTo -- docker-daemon:bits:latest

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

# ------------------------------------------------------------------------------
# Operations

# Interactive shell on compute as the ci user
[group('ops')]
[no-exit-message]
ssh:
    @exec ssh -tt compute "cd /tmp && exec sudo -Hu ci XDG_RUNTIME_DIR=/run/user/\$(id -u ci) TERM=$TERM bash -c 'cd && exec bash'"

# Run podman commands on compute as the ci user
[group('ops')]
[no-exit-message]
podman *args:
    @exec ssh -tt compute "cd /tmp && exec sudo -Hu ci TERM=$TERM XDG_RUNTIME_DIR=/run/user/\$(id -u ci) podman {{ args }}"

# Run systemctl commands on compute as the ci user
[group('ops')]
[no-exit-message]
systemctl *args:
    @exec ssh -tt compute "cd /tmp && exec sudo -Hu ci TERM=$TERM XDG_RUNTIME_DIR=/run/user/\$(id -u ci) systemctl --user {{ args }}"

# View logs for a container
[group('ops')]
[no-exit-message]
logs container *args:
    @exec ssh -tt compute "exec sudo journalctl --output=cat CONTAINER_NAME={{ container }} {{ args }}"

# ------------------------------------------------------------------------------
# CI

forgejo_api := "https://code.invetica.team/api/v1"
forgejo_repo := "jcf/bits"

# Authenticate with Forgejo UI for log access
[group('ci')]
ci-login:
    fj-ex auth login --host code.invetica.team

# Show the latest CI run status
[group('ci')]
ci-status:
    #!/usr/bin/env bash
    set -euo pipefail
    token=$(op read "op://Employee/Forgejo/tokens/everything")
    curl -s -H "Authorization: token $token" \
        "{{ forgejo_api }}/repos/{{ forgejo_repo }}/actions/runs" \
    | jq -r '[.workflow_runs[] | select(.created != null)] | sort_by(.created) | reverse | .[0] | "Run #\(.index_in_repo): \(.title)\nStatus: \(.status)\nURL: \(.html_url)"'

# List recent CI runs
[group('ci')]
ci-runs limit="10":
    #!/usr/bin/env bash
    set -euo pipefail
    token=$(op read "op://Employee/Forgejo/tokens/everything")
    curl -s -H "Authorization: token $token" \
        "{{ forgejo_api }}/repos/{{ forgejo_repo }}/actions/runs" \
    | jq -r --argjson n "{{ limit }}" \
        '[.workflow_runs[] | select(.created != null)] | sort_by(.created) | reverse | .[:$n][] | "#\(.index_in_repo)\t\(.status)\t\(.title)"'

# Show jobs for a CI run (index is first column, use with ci-logs)
[group('ci')]
ci-jobs run:
    fj-ex actions jobs --run-index {{ run }}

# Show failed jobs from the latest CI run
[group('ci')]
ci-failures:
    #!/usr/bin/env bash
    set -euo pipefail
    token=$(op read "op://Employee/Forgejo/tokens/everything")

    # Get latest run number (sorted by created date)
    run=$(curl -s -H "Authorization: token $token" \
        "{{ forgejo_api }}/repos/{{ forgejo_repo }}/actions/runs" \
    | jq -r '[.workflow_runs[] | select(.created != null)] | sort_by(.created) | reverse | .[0].index_in_repo')

    echo "Run #$run failures:"
    curl -s -H "Authorization: token $token" \
        "{{ forgejo_api }}/repos/{{ forgejo_repo }}/actions/tasks" \
    | jq -r --arg run "$run" \
        '.workflow_runs[] | select(.run_number == ($run | tonumber) and .status == "failure") | "  - \(.name)"'

    echo ""
    echo "View logs: https://code.invetica.team/{{ forgejo_repo }}/actions/runs/$run"

# Fetch logs for a CI run or specific job (requires fj-ex to be authenticated)
[group('ci')]
ci-logs run job="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -z "{{ job }}" ]]; then
        fj-ex actions logs run --run-index {{ run }}
    else
        fj-ex actions logs job --run-index {{ run }} --job-index {{ job }}
    fi
