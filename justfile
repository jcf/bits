machine := "nix"

[group('host')]
_default:
    @just --list

[group('host')]
fmt:
    treefmt

[group('host')]
run *args:
    orbctl run --machine {{ machine }} {{ args }}

[group('vm')]
setup:
    devenv shell echo "🚀 Development environment ready!"
    pnpm install

[group('vm')]
dev:
    pnpm dev
