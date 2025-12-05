# Terraform will panic if it encounters `PGSERVICEFILE`.

unexport PGSERVICEFILE

plan_dir := justfile_directory() / ".terraform-plans"

_default:
    @just fmt
    @just build
    @just lint
    @just test

# ------------------------------------------------------------------------------
# Setup

# Create self-signed SSL certificates via mkcert
[group('setup')]
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
[group('setup')]
setup:
    @just mkcert
    @just hetzner-contexts
    devenv shell true
    pnpm install
    @echo -e "\n✅ {{ BOLD }}Setup complete!{{ BOLD }}"

# Upsert Bits contexts
[group('setup')]
hetzner-contexts:
    #!/usr/bin/env zsh
    for ctx in bits-{dev,prod}; do
      hcloud context list | grep "$ctx" &>/dev/null ||
        hcloud context create $ctx
    done

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
# Serve

# Run Dioxus in solo mode
[group('serve')]
solo:
    dx serve --platform web --fullstack true --port 3000 --package bits-solo

# Run Dioxus in co-located mode
[group('serve')]
colo:
    dx serve --platform web --fullstack true --port 3000 --package bits-colo

# Run Dioxus in co-located mode
[group('serve')]
dev: colo

# Watch source code for Tailwind classes
[group('serve')]
tailwind dir:
    pnpm --filter @bits/tailwind \
        exec tailwindcss \
            --watch=always \
            --input ../../{{ dir }}/tailwind.css \
            --output ../../{{ dir }}/assets/app.css

# Run the marketing site
[group('serve')]
www:
    pnpm --filter @bits/www dev

# ------------------------------------------------------------------------------
# Database

# Start an interactive psql session connected to the local development database
[group('db')]
psql *args:
    PGPASSWD=please psql \
        --host=localhost \
        --port=5432 \
        --username=bits \
        --dbname=bits_dev \
        {{ args }}

# Run database migrations
[group('db')]
db-migrate:
    sqlx migrate run

# Create a new migration
[group('db')]
db-migration name:
    sqlx migrate add {{ name }}

# Seed the database with test data
[group('db')]
db-seed:
    cargo run --bin bits-seed

# Reset all development PostgreSQL state
[group('db')]
db-reset:
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
# Quality

# Format project files
[group('quality')]
fmt:
    treefmt

# Format rsx expressions via stdin (please use `just fmt`)
[group('quality')]
_fmt-dx *_args:
    dx fmt --file -

# Fix errors within Bits
[group('quality')]
fix:
    cargo fix --lib -p bits-app

# Run checks
[group('quality')]
check:
    cargo check

# Run lints
[group('quality')]
lint:
    env RUSTFLAGS="-D warnings" cargo clippy -- -D warnings

# ------------------------------------------------------------------------------
# Test

# Run units tests
[group('test')]
unit:
    env RUSTFLAGS="-D warnings" cargo nextest run --workspace --exclude bits-e2e --features server

# Run integration tests
[group('test')]
integrate:
    env RUSTFLAGS="-D warnings" cargo nextest run --package bits-e2e --features server

# Run unit and integration tests
[group('test')]
test:
    @just unit
    @just integrate

# ------------------------------------------------------------------------------
# Build

# Build fullstack web packages
[group('build')]
build:
    env RUSTFLAGS="-D warnings" dx build --fullstack true --platform web --package bits-solo
    env RUSTFLAGS="-D warnings" dx build --fullstack true --platform web --package bits-colo

# Bundle a release build via dioxus-cli
[group('build')]
release:
    dx bundle --release

# Clear build caches
[group('build')]
clean:
    cargo clean

# ------------------------------------------------------------------------------
# Workflows

# Verify and push changes
[group('workflows')]
push:
    @just test
    git push

# ------------------------------------------------------------------------------
# Infrastructure

[group('infra')]
_terraform dir *args:
    op run -- terraform -chdir={{ justfile_directory() }}/iac/{{ dir }} {{ args }}

# Initialize one or all Terraform projects
[group('infra')]
init dir *args:
    @just _terraform {{ dir }} init {{ args }}

# Plan one or all Terraform projects
[group('infra')]
plan dir:
    @mkdir -p {{ plan_dir }}
    @just _terraform {{ dir }} plan -out {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan

# Apply one or all Terraform projects
[group('infra')]
apply dir:
    @just _terraform {{ dir }} apply {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan
    rm {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan

# Import into a Terraform project
[group('infra')]
import dir *args:
    @just _terraform {{ dir }} import {{ args }}

# Interact with outputs one or all Terraform projects
[group('infra')]
output dir *args:
    @just _terraform {{ dir }} output {{ args }}

# Build NixOS image snapshot from flake
[group('infra')]
snapshot-build snapshot_name="":
    #!/usr/bin/env bash
    set -e

    # Generate snapshot name if not provided
    if [[ -z "{{ snapshot_name }}" ]]; then
        snapshot_name="test-$(date +%Y%m%d-%H%M%S)"
    else
        snapshot_name="{{ snapshot_name }}"
    fi

    echo "Building NixOS snapshot: ${snapshot_name}"
    echo "This will use the flake-based NixOS configuration from .#nixosConfigurations.bits-prod"

    # Build the Packer script from our Nix expression
    op run -- nix-build nix/images/build-snapshot.nix \
      --arg hcloud-token "\"$HCLOUD_TOKEN\"" \
      --arg snapshot-name "\"${snapshot_name}\"" \
      --arg snapshot-description "\"NixOS 25.05 snapshot for Bits platform\"" \
      --arg server-type "\"cx23\"" \
      --arg server-location "\"nbg1\""

    # Execute the Packer build
    op run -- ./result/bin/build-snapshot

    # Print snapshot name for scripting
    echo "${snapshot_name}"

# List NixOS image snapshots
[group('infra')]
snapshot-list:
    @hcloud image list -t snapshot -o columns=id,name,created,architecture

# Delete NixOS image snapshot
[group('infra')]
snapshot-delete name:
    @hcloud image delete "{{ name }}"

# ------------------------------------------------------------------------------
# Deploy

# Build Rust binary with Nix
[group('deploy')]
nixos-build:
    nix build .#bits --print-build-logs

# Build NixOS snapshot with baked-in application
[group('deploy')]
nixos-snapshot:
    #!/usr/bin/env zsh
    set -e
    sha=$(git rev-parse --short HEAD)
    snapshot_name="nixos-25.05-${sha}"

    echo "📦 Building NixOS snapshot with Bits application..."
    echo "Git SHA: ${sha}"
    echo "Snapshot: ${snapshot_name}"
    echo ""

    # Build and create snapshot via Packer
    just snapshot-build "${snapshot_name}"

    # Update Terraform variable
    echo "nixos_snapshot_name = \"${snapshot_name}\"" > iac/platform/terraform.auto.tfvars
    echo ""
    echo "✅ Built snapshot: ${snapshot_name}"
    echo "ℹ️  Next: Review with 'just plan platform', then deploy with 'just apply platform'"

# Deploy NixOS snapshot to production (blue/green deployment)
[group('deploy')]
nixos-deploy:
    #!/usr/bin/env zsh
    set -e

    echo "🚀 Starting NixOS deployment..."

    # Build snapshot (updates terraform.auto.tfvars)
    just nixos-snapshot

    # Plan changes
    echo ""
    echo "📋 Planning Terraform changes..."
    just plan platform

    # Confirm deployment
    echo ""
    echo -n "{{ BOLD }}Deploy to production? This will create a new server from the snapshot. (y/N): {{ NORMAL }}"
    read confirm

    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        just apply platform
        echo "✅ Deployment complete!"
        echo "ℹ️  New server is running. Old server will be automatically destroyed."
    else
        echo "❌ Deployment cancelled"
        exit 1
    fi

# Rollback to previous NixOS snapshot
[group('deploy')]
nixos-rollback snapshot_name:
    #!/usr/bin/env zsh
    set -e

    echo "⏪ Rolling back to snapshot: {{ snapshot_name }}"

    # Update Terraform variable
    echo "nixos_snapshot_name = \"{{ snapshot_name }}\"" > iac/platform/terraform.auto.tfvars

    # Plan and apply
    just plan platform
    echo ""
    echo -n "{{ BOLD }}Confirm rollback? (y/N): {{ NORMAL }}"
    read confirm

    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        just apply platform
        echo "✅ Rollback complete!"
    else
        echo "❌ Rollback cancelled"
        exit 1
    fi
