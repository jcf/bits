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

cyan=$(tput setaf 6)
yellow=$(tput setaf 3)
green=$(tput setaf 2)
red=$(tput setaf 1)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}==>${reset} ${bold}$*${reset}"
}

ok() {
  echo "${green}ok:${reset} $*"
}

warn() {
  echo "${yellow}wait:${reset} $*"
}

err() {
  echo >&2 "${red}error:${reset} $*"
}

# Wait for an image to be available in the registry with exponential backoff
wait_for_image() {
  local img=$1
  local auth=$2
  local max_attempts=6
  local delay=2

  say "Checking $img..."

  for ((attempt = 1; attempt <= max_attempts; attempt++)); do
    if skopeo inspect --authfile "$auth" "docker://$img" >/dev/null 2>&1; then
      ok "Found"
      return 0
    fi
    if ((attempt < max_attempts)); then
      warn "Attempt $attempt/$max_attempts, retrying in ${delay}s..."
      sleep "$delay"
      delay=$((delay * 2))
    fi
  done

  err "$img not available after $max_attempts attempts"
  return 1
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

say "Logging in to $registry..."
echo "$REGISTRY_TOKEN" |
  podman login "$registry" -u "$REGISTRY_USER" \
    --authfile "$authfile" --password-stdin

# Build list of arch-specific images
arch_images=()
for arch in "${archs[@]}"; do
  arch_images+=("$image:$version-$arch")
done

# Wait for all arch images to be available
for img in "${arch_images[@]}"; do
  wait_for_image "$img" "$authfile"
done

# Helper to create, populate and push a manifest
push_manifest() {
  local tag=$1
  say "Creating manifest $image:$tag..."
  podman manifest create "$image:$tag"
  for img in "${arch_images[@]}"; do
    podman manifest add --authfile "$authfile" "$image:$tag" "$img"
  done
  podman manifest push --authfile "$authfile" "$image:$tag" "docker://$image:$tag"
  ok "Pushed $image:$tag"
}

push_manifest "$version"
push_manifest "latest"
