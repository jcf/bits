#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: push-image.sh <package-name> <registry/image> <tag> [deps-lock-path]"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  push-image.sh bits-container-amd64 ghcr.io/jcf/bits 20260225-abc123-amd64"
  echo >&2 "  push-image.sh bits-container-amd64 ghcr.io/jcf/bits 20260225-abc123-amd64 /tmp/deps-lock.json"
  echo >&2 ""
  echo >&2 "Environment variables:"
  echo >&2 "  REGISTRY_USER   Username for registry authentication"
  echo >&2 "  REGISTRY_TOKEN  Token for registry authentication"
}

if [[ $# -lt 3 || $# -gt 4 ]]; then
  usage
  exit 22
fi

package=$1
image=$2
tag=$3
deps_lock=${4:-}

cyan=$(tput setaf 6)
green=$(tput setaf 2)
dim=$(tput dim)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}==>${reset} ${bold}$*${reset}"
}

ok() {
  echo "${green}ok:${reset} $*"
}

registry="${image%%/*}"
authfile="${REGISTRY_AUTH_FILE:-$(mktemp -d)/auth.json}"

say "Pushing to ${registry}..."

echo "$REGISTRY_TOKEN" | skopeo login "$registry" -u "$REGISTRY_USER" --authfile "$authfile" --password-stdin

nix_args=()
if [[ -n $deps_lock ]]; then
  nix_args+=(--override-input deps-lock "path:$deps_lock")
fi

nix run ".#$package.copyTo" "${nix_args[@]}" -- \
  --dest-authfile "$authfile" \
  "docker://$image:$tag"

ok "Pushed"
echo ""
echo "${dim}image${reset}"
echo "$image:$tag"
