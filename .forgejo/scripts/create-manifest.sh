#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: create-manifest.sh <image> <version> <arch>..."
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  create-manifest.sh ghcr.io/jcf/bits 20260225-abc123 amd64 arm64"
  echo >&2 ""
  echo >&2 "Environment variables:"
  echo >&2 "  REGISTRY_USER   Username for registry authentication"
  echo >&2 "  REGISTRY_TOKEN  Token for registry authentication"
}

if [[ $# -lt 3 ]]; then
  usage
  exit 22 # EINVAL
fi

image=$1
version=$2
shift 2
archs=("$@")

registry="${image%%/*}"
authfile="${REGISTRY_AUTH_FILE:-$(mktemp -d)/auth.json}"

echo "$REGISTRY_TOKEN" | podman login "$registry" -u "$REGISTRY_USER" --authfile "$authfile" --password-stdin

# Build list of arch-specific images
arch_images=()
for arch in "${archs[@]}"; do
  arch_images+=("$image:$version-$arch")
done

# Create versioned manifest
podman manifest create "$image:$version" "${arch_images[@]}"
podman manifest push --authfile "$authfile" "$image:$version" "docker://$image:$version"
echo "Pushed: $image:$version"

# Create latest manifest
podman manifest create "$image:latest" "${arch_images[@]}"
podman manifest push --authfile "$authfile" "$image:latest" "docker://$image:latest"
echo "Pushed: $image:latest"
