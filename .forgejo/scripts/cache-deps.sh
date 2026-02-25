#!/usr/bin/env bash
set -eu

deps_lock=${1:-}

cyan=$(tput setaf 6)
green=$(tput setaf 2)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}==>${reset} ${bold}$*${reset}"
}

ok() {
  echo "${green}ok:${reset} $*"
}

say "Building deps cache..."

nix_args=(--no-link --print-out-paths)
if [[ -n $deps_lock ]]; then
  nix_args+=(--override-input deps-lock "path:$deps_lock")
fi

cache_path=$(nix build .#bits-deps-cache "${nix_args[@]}")

say "Pushing to Attic..."
echo ""
echo "$cache_path"
echo ""
attic push invetica:invetica "$cache_path"

ok "Deps cached"
