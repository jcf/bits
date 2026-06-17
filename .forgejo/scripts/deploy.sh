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
green=$(tput setaf 2)
red=$(tput setaf 1)
bold=$(tput bold)
reset=$(tput sgr0)

say() { echo >&2 "${cyan}→${reset} ${bold}$*${reset}"; }
ok() { echo >&2 "${green}ok:${reset} $*"; }
err() { echo >&2 "${red}error:${reset} $*"; }

target_user="bits"
xdg_runtime="/run/user/$(id -u "$target_user")"
quadlet_dir="/var/lib/${target_user}/.config/containers/systemd/${service}"

# NixOS setuid wrapper. The Forgejo runner's PATH excludes
# /run/wrappers/bin, so `sudo` isn't found without the absolute path.
sudo=/run/wrappers/bin/sudo

# Run a command as the target user with their systemd-user runtime
# directory in scope, so rootless podman finds its socket and
# systemctl --user talks to the right instance.
as_target() {
  $sudo -u "$target_user" XDG_RUNTIME_DIR="$xdg_runtime" "$@"
}

say "Pinning image to $image..."
sed -i "s|Image=code.invetica.team/jcf/bits:.*|Image=$image|" deploy/bits.container

say "Installing quadlet files to $quadlet_dir..."
as_target install -d -m 0755 "$quadlet_dir"
as_target install -m 0644 \
  deploy/*.container deploy/*.network deploy/*.volume deploy/*.sql \
  "$quadlet_dir/"

say "Logging in to $registry..."
as_target podman login -u "$REGISTRY_USER" -p "$REGISTRY_TOKEN" "$registry"

say "Pulling $image..."
as_target podman pull "$image"

say "Pruning unused images..."
as_target podman image prune -af

say "Waiting for service to start..."
max_attempts=12
delay=5

for ((attempt = 1; attempt <= max_attempts; attempt++)); do
  if as_target systemctl --user is-active --quiet "$service.service"; then
    ok "Service running"
    exit 0
  fi
  if ((attempt < max_attempts)); then
    echo "Attempt $attempt/$max_attempts, retrying in ${delay}s..."
    sleep "$delay"
  fi
done

err "Service not running after $max_attempts attempts"
as_target systemctl --user status "$service.service" || true
exit 1
