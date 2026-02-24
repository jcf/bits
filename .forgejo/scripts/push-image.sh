#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: push-image.sh <image-path> <image> <tag>"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  push-image.sh /nix/store/...-bits.tar.gz ghcr.io/jcf/bits 20260225-abc123-arm64"
  echo >&2 ""
  echo >&2 "Environment variables:"
  echo >&2 "  REGISTRY_USER   Username for registry authentication"
  echo >&2 "  REGISTRY_TOKEN  Token for registry authentication"
}

if [[ $# -ne 3 ]]; then
  usage
  exit 22 # EINVAL
fi

image_path=$1
image=$2
tag=$3

registry="${image%%/*}"
authfile="${REGISTRY_AUTH_FILE:-$(mktemp -d)/auth.json}"

echo "$REGISTRY_TOKEN" | skopeo login "$registry" -u "$REGISTRY_USER" --authfile "$authfile" --password-stdin
skopeo copy --authfile "$authfile" "docker-archive:$image_path" "docker://$image:$tag"
echo "Pushed: $image:$tag"
