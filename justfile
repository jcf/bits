_default:
    @just --list

mkcert:
    #!/usr/bin/env zsh
    mkcert -install
    mkdir -p {{ justfile_directory() }}/certs
    cd {{ justfile_directory() }}/certs

    # Generate wildcard cert for *.invetica.dev (covers edit.invetica.dev, page.invetica.dev)
    if [[ ! -f "_wildcard.invetica.dev.pem" ]]; then
      mkcert '*.invetica.dev'
    fi

    # Generate wildcard cert for *.page.invetica.dev (covers customer subdomains)
    if [[ ! -f "_wildcard.page.invetica.dev.pem" ]]; then
      mkcert '*.page.invetica.dev'
    fi

    echo >&2 "🔒 Wildcard certificates ready!"

setup:
    @just mkcert
    devenv shell echo "🚀 Development environment ready!"
    pnpm install

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

fmt:
    treefmt

dev:
    pnpm dev
