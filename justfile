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

fmt:
    treefmt

dev:
    pnpm dev
