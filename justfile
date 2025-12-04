# Terraform will panic if it encounters `PGSERVICEFILE`.

unexport PGSERVICEFILE

plan_dir := justfile_directory() / ".terraform-plans"

_default:
    @just fmt
    @just build
    @just lint
    @just test

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

# Create a new prompt
[group('docs')]
prompt +title:
    #!/usr/bin/env zsh
    timestamp=$(date +%Y%m%d%H%M%S)
    normalized=$(echo "{{ title }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    filename=".claude/prompts/${timestamp}-${normalized}.org"
    mkdir -p .claude/prompts
    cat > "$filename" <<EOF
    #+title:    {{ title }}
    #+date:     $(date +%Y-%m-%d)
    #+property: header-args :dir ../..
    EOF
    echo "💭 {{ BOLD }}Created \"$filename\"{{ NORMAL }}."

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

# Run Dioxus in solo mode
[group('dev')]
solo:
    dx serve --platform web --fullstack true --port 3000 --package bits-solo

# Run Dioxus in co-located mode
[group('dev')]
colo:
    dx serve --platform web --fullstack true --port 3000 --package bits-colo

# Run Dioxus in co-located mode
[group('dev')]
dev: colo

# Watch source code for Tailwind classes
[group('dev')]
tailwind dir:
    pnpm --filter @bits/tailwind \
        exec tailwindcss \
            --watch=always \
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

# Format rsx expressions via stdin (please use `just fmt`)
[group('dev')]
_fmt-dx *_args:
    dx fmt --file -

# Build fullstack web packages
[group('dev')]
build:
    env RUSTFLAGS="-D warnings" dx build --fullstack true --platform web --package bits-solo
    env RUSTFLAGS="-D warnings" dx build --fullstack true --platform web --package bits-colo

# Run checks
[group('dev')]
check:
    cargo check

# Run lints
[group('dev')]
lint:
    env RUSTFLAGS="-D warnings" cargo clippy -- -D warnings

# Run units tests
[group('dev')]
unit:
    env RUSTFLAGS="-D warnings" cargo nextest run --features server

# Run integration tests
[group('dev')]
integrate:
    env RUSTFLAGS="-D warnings" cargo nextest run --package bits-e2e --features server

# Run unit and integration tests
[group('dev')]
test:
    @just unit
    @just integrate

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

# Seed the database with test data
[group('postgres')]
seed:
    cargo run --bin bits-seed

# Delete all development PostgreSQL state
[group('postgres')]
db-destroy:
    #!/usr/bin/env zsh
    dir=".devenv/state/postgres"
    [[ ! -d "$dir" ]] && exit

    echo -n "{{ BOLD }}Are you sure you want to delete {{ YELLOW }}${dir}{{ NORMAL }}{{ BOLD }}? (y/N): {{ NORMAL }}"
    read response

    if [[ "$response" =~ ^[Yy]$ ]]; then
        rm -r .devenv/state/postgres/
        echo >&2 "🔥 PostgreSQL state deleted."
    fi

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

# Import into a Terraform project
[group('iac')]
import dir *args:
    @just _terraform {{ dir }} import {{ args }}

# Interact with outputs one or all Terraform projects
[group('iac')]
output dir *args:
    @just _terraform {{ dir }} output {{ args }}
