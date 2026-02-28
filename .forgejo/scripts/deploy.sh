#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: deploy.sh <service> <image>"
  echo >&2 ""
  echo >&2 "Deploy quadlet files and pull container image. The bits.container"
  echo >&2 "quadlet is updated to reference the exact image tag, triggering a"
  echo >&2 "restart via systemd path unit."
  echo >&2 ""
  echo >&2 "Environment:"
  echo >&2 "  REGISTRY_USER   Registry username"
  echo >&2 "  REGISTRY_TOKEN  Registry password/token"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  deploy.sh bits code.invetica.team/jcf/bits:20260228-abc123"
}

if [[ $# -ne 2 ]]; then
  usage
  exit 22
fi

service=$1
image=$2
registry=$(echo "$image" | cut -d/ -f1)

cyan=$(tput setaf 6)
bold=$(tput bold)
reset=$(tput sgr0)

say() { echo >&2 "${cyan}==>${reset} ${bold}$*${reset}"; }

quadlet_dir="/var/lib/ci/.config/containers/systemd/bits"

say "Installing quadlet files..."
mkdir -p "$quadlet_dir"
rm -f "$quadlet_dir"/*.container "$quadlet_dir"/*.network "$quadlet_dir"/*.volume "$quadlet_dir"/*.sql
cp deploy/*.container deploy/*.network deploy/*.volume deploy/*.sql "$quadlet_dir/"

say "Pinning image to $image..."
sed -i "s|Image=code.invetica.team/jcf/bits:.*|Image=$image|" "$quadlet_dir/bits.container"

say "Logging in to $registry..."
podman login -u "$REGISTRY_USER" -p "$REGISTRY_TOKEN" "$registry"

say "Pulling $image..."
podman pull "$image"

say "Pruning unused images..."
podman image prune -af

say "Deployment complete"
