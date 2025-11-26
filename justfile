# Terraform will panic if it encounters `PGSERVICEFILE`.

unexport PGSERVICEFILE

plan_dir := justfile_directory() / ".terraform-plans"

_default:
    @just fmt
    @just build
    @just lint
    @just test
    @just integration

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

# Setup a local development environment
[group('dev')]
setup:
    @just mkcert
    devenv shell true
    pnpm install
    @echo -e "\n✅ {{ BOLD }}Setup complete!{{ BOLD }}"

# Clear build caches
[group('dev')]
clean:
    cargo clean

# Run Dioxus in single-tenant mode
[group('dev')]
serve:
    dx serve --platform web --fullstack true --port 3000 --package bits-solo

# Run Dioxus in multi-tenant mode
[group('dev')]
multi:
    dx serve --platform web --fullstack true --port 3000 --package bits-colo

# Watch source code for Tailwind classes
[group('dev')]
tailwind dir:
    pnpm --filter @bits/tailwind \
        exec tailwindcss \
            --watch \
            --input ../../{{ dir }}/tailwind.css \
            --output ../../{{ dir }}/assets/app.css

# Run the marketing site
[group('dev')]
www:
    pnpm --filter @bits/www dev

# Fix errors within Bits
[group('dev')]
fix:
    cargo fix --lib -p bits-app

# Format project files
[group('dev')]
fmt:
    treefmt

# Build fullstack web packages
[group('dev')]
build:
    dx build --fullstack true --platform web --package bits-solo
    dx build --fullstack true --platform web --package bits-colo

# Run checks
[group('dev')]
check:
    cargo check

# Run lints
[group('dev')]
lint:
    cargo clippy

# Run tests
[group('dev')]
test:
    cargo nextest run --features server

# Run integration tests
[group('dev')]
integration:
    cargo nextest run --package bits-colo --features server --test integration

# Verify and push changes
[group('dev')]
push:
    @just test
    git push

[group('dev')]
release:
    dx bundle --release

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

# Run database migrations
[group('postgres')]
migrate:
    sqlx migrate run

# Create a new migration
[group('postgres')]
migration name:
    sqlx migrate add {{ name }}

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
