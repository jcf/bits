#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: push-image.sh <copy-to-script> <registry/image> <tag>"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  push-image.sh /nix/store/...-copy-to ghcr.io/jcf/bits 20260225-abc123-amd64"
  echo >&2 ""
  echo >&2 "Environment variables:"
  echo >&2 "  REGISTRY_USER   Username for registry authentication"
  echo >&2 "  REGISTRY_TOKEN  Token for registry authentication"
}

if [[ $# -ne 3 ]]; then
  usage
  exit 22
fi

copy_to=$1
image=$2
tag=$3

cyan=$(tput setaf 6)
green=$(tput setaf 2)
dim=$(tput dim)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}→${reset} ${bold}$*${reset}"
}

ok() {
  echo "${green}ok:${reset} $*"
}

registry="${image%%/*}"
authfile="${REGISTRY_AUTH_FILE:-$(mktemp -d)/auth.json}"

say "Pushing to ${registry}..."

echo "$REGISTRY_TOKEN" | skopeo login "$registry" -u "$REGISTRY_USER" --authfile "$authfile" --password-stdin

# Use the pre-built copyTo script directly (no flake re-evaluation)
"$copy_to" \
  --dest-authfile "$authfile" \
  --dest-compress-format zstd \
  "docker://$image:$tag"

ok "Pushed"
echo ""
echo "${dim}image${reset}"
echo "$image:$tag"
