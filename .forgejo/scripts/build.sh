#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: build.sh <package-name> [deps-lock-path]"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  build.sh bits-ci-amd64"
  echo >&2 "  build.sh bits-container-amd64 /tmp/deps-lock.json"
  echo >&2 ""
  echo >&2 "When GITHUB_OUTPUT is set, writes image_path and tag to it."
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  exit 22
fi

package=$1
deps_lock=${2:-}

cyan=$(tput setaf 6)
green=$(tput setaf 2)
red=$(tput setaf 1)
dim=$(tput dim)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}→${reset} ${bold}$*${reset}"
}

ok() {
  echo "${green}ok:${reset} $*"
}

err() {
  echo "${red}error:${reset} ${bold}$*${reset}" >&2
}

# Compute tag from git state (before any modifications)
tag="$(date -u +%Y%m%d)-$(git rev-parse --short HEAD)"

# Verify we're on a clean tree
if [[ -n $(git status --porcelain) ]]; then
  err "Refusing to build from dirty worktree"
  git status --short >&2
  exit 1
fi

say "Building ${package}..."

nix_args=(--no-link --print-out-paths --quiet)
if [[ -n $deps_lock ]]; then
  nix_args+=(--override-input deps-lock "path:$deps_lock")
fi

image_path=$(nix build ".#$package" "${nix_args[@]}")

if [[ -z $image_path || ! -e $image_path ]]; then
  err "Build failed or output path is invalid"
  exit 1
fi

# Build the copyTo script (separate derivation from the image)
# nix2container uses writeShellApplication, so script is at bin/copy-to
copy_to=$(nix build ".#$package.copyTo" "${nix_args[@]}")/bin/copy-to

ok "Build complete"
echo ""
echo "${dim}image_path${reset}"
echo "$image_path"
echo ""
echo "${dim}copy_to${reset}"
echo "$copy_to"
echo ""
echo "${dim}tag${reset}"
echo "$tag"

if [[ -n ${GITHUB_OUTPUT:-} ]]; then
  echo "image_path=$image_path" >>"$GITHUB_OUTPUT"
  echo "copy_to=$copy_to" >>"$GITHUB_OUTPUT"
  echo "tag=$tag" >>"$GITHUB_OUTPUT"
fi
