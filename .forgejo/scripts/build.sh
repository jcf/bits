#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: build.sh <output-name>"
}

if [[ $# -ne 1 ]]; then
  usage
  exit 22 # EINVAL
fi

output=$1

# Build the container
devenv build "outputs.$output"

# Output the image tag from passthru
tag=$(nix eval ".#devenv.config.outputs.$output.passthru.imageTag" --raw 2>&1) || {
  echo >&2 "Error: Failed to get imageTag from outputs.$output"
  echo >&2 "$tag"
  exit 1
}

if [[ -z "$tag" ]]; then
  echo >&2 "Error: imageTag is empty for outputs.$output"
  exit 1
fi

echo "$tag"
