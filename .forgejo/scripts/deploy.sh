#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: deploy.sh <service> <image>"
  echo >&2 ""
  echo >&2 "Deploy quadlet files and update service."
  echo >&2 ""
  echo >&2 "Environment:"
  echo >&2 "  REGISTRY_USER   Registry username"
  echo >&2 "  REGISTRY_TOKEN  Registry password/token"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  deploy.sh bits code.invetica.team/jcf/bits:latest"
}

if [[ $# -ne 2 ]]; then
  usage
  exit 22
fi

service=$1
image=$2
registry=$(echo "$image" | cut -d/ -f1)

# Set up environment for systemd --user
export XDG_RUNTIME_DIR="/run/user/$(id -u)"

cyan=$(tput setaf 6)
red=$(tput setaf 1)
bold=$(tput bold)
reset=$(tput sgr0)

say() { echo >&2 "${cyan}==>${reset} ${bold}$*${reset}"; }
err() { echo >&2 "${red}${bold}error:${reset} ${bold}$*${reset}"; }

quadlet_dir="$HOME/.config/containers/systemd"

say "Installing quadlet files..."
mkdir -p "$quadlet_dir"
cp deploy/*.container deploy/*.network deploy/*.volume "$quadlet_dir/"

say "Reloading systemd..."
systemctl --user daemon-reload

say "Logging in to $registry..."
podman login -u "$REGISTRY_USER" -p "$REGISTRY_TOKEN" "$registry"

say "Pulling $image..."
podman pull "$image"

say "Restarting $service..."
systemctl --user restart "$service"

say "Waiting for $service..."
for i in {1..24}; do
  if systemctl --user is-active --quiet "$service"; then
    say "Deployment complete"
    exit 0
  fi
  sleep 5
done

err "Service $service failed to start"
systemctl --user status "$service" --no-pager >&2 || true
exit 1
