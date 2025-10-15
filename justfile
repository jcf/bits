domain := "bits.test"
plan_dir := justfile_directory() / ".terraform-plans"

_default:
    @just --list

# Create self-signed SSL certificates via mkcert
[group('setup')]
mkcert:
    #!/usr/bin/env zsh
    set -e

    mkcert -install
    mkdir -p {{ justfile_directory() }}/certs
    cd {{ justfile_directory() }}/certs

    # Generate wildcard cert for *.test (covers edit.test, page.test)
    if [[ ! -f "_wildcard.{{ domain }}.pem" ]]; then
      mkcert '*.{{ domain }}'
    fi

    # Generate wildcard cert for customer subdomains
    if [[ ! -f "_wildcard.page.{{ domain }}.pem" ]]; then
      mkcert '*.page.{{ domain }}'
    fi

    echo >&2 "🔒 Wildcard certificates ready!"

# Setup a local development environment
[group('setup')]
setup:
    @just mkcert
    devenv shell echo "🚀 Development environment ready!"
    pnpm install

# Create a new decision record
[group('docs')]
decide +title:
    #!/usr/bin/env bash
    timestamp=$(date +%Y%m%d%H%M%S)
    normalized=$(echo "{{ title }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    capitalized=$(echo "{{ title }}" | tr '[:upper:]' '[:lower:]' | sed 's/./\U&/')
    author="$(git config user.name) <$(git config user.email)>"
    filename="decisions/${timestamp}-${normalized}.org"
    cat > "$filename" <<EOF
    #+title:  ${capitalized}
    #+author: ${author}
    #+date:   $(date +%Y-%m-%d)
    EOF
    echo "🎯 {{ BOLD }}Created \"$filename\"{{ NORMAL }}."

# Format project files
[group('dev')]
fmt:
    treefmt

# Fire up a local development environment
[group('dev')]
dev:
    pnpm dev

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
