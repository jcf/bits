#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: tag-image.sh <image-path> <arch>"
}

if [[ $# -ne 2 ]]; then
  usage
  exit 22 # EINVAL
fi

image_path=$1
arch=$2

if [[ ! -f $image_path ]]; then
  echo >&2 "Error: Image not found: $image_path"
  exit 1
fi

inspect_output=$(skopeo inspect "docker-archive:$image_path") || {
  echo >&2 "Error: skopeo inspect failed for $image_path"
  exit 1
}

repo_tag=$(echo "$inspect_output" | jq -r '.RepoTags[0]')
if [[ -z $repo_tag || $repo_tag == "null" ]]; then
  echo >&2 "Error: No RepoTags found in image"
  echo >&2 "skopeo output: $inspect_output"
  exit 1
fi

version=$(echo "$repo_tag" | cut -d: -f2)
if [[ -z $version || $version == "null" ]]; then
  echo >&2 "Error: Could not extract version from tag: $repo_tag"
  exit 1
fi

echo "Version: $version"
echo "version=$version" >>"$GITHUB_OUTPUT"
