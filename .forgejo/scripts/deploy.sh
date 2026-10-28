#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: deploy.sh <service> <image>"
  echo >&2 ""
  echo >&2 "Blue/green deploy. Starts the idle colour on the new image, waits for"
  echo >&2 "its /healthz, points bits-proxy at it, then stops the old colour."
  echo >&2 "Supporting services restart only when their definitions change."
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
unit_dir="/var/lib/${target_user}/.config/systemd/user"
rendered="deploy/rendered"

# NixOS setuid wrapper. The Forgejo runner's PATH excludes
# /run/wrappers/bin, so `sudo` isn't found without the absolute path.
sudo=/run/wrappers/bin/sudo

# Run a command as the target user with their systemd-user runtime
# directory in scope, so rootless podman finds its socket and
# systemctl --user talks to the right instance.
as_target() {
  $sudo -u "$target_user" XDG_RUNTIME_DIR="$xdg_runtime" "$@"
}

user_systemctl() {
  as_target systemctl --user "$@"
}

port_of() {
  case $1 in
    blue) echo 3001 ;;
    green) echo 3002 ;;
  esac
}

render() {
  local colour=$1 template=$2 out=$3
  sed -e "s|@COLOUR@|$colour|g" \
    -e "s|@PORT@|$(port_of "$colour")|g" \
    -e "s|@IMAGE@|$image|g" \
    "$template" >"$out"
}

# The live colour is whichever port the installed proxy forwards to.
live_port=$(user_systemctl cat "$service-proxy.service" 2>/dev/null |
  sed -n 's/^ExecStart=.*127\.0\.0\.1:\([0-9]*\)$/\1/p' || true)
case $live_port in
  3001) live=blue idle=green ;;
  3002) live=green idle=blue ;;
  *) live="" idle=blue ;;
esac
idle_port=$(port_of "$idle")

say "Live colour: ${live:-none}; deploying $image to $idle..."
mkdir -p "$rendered"
render "$idle" "deploy/$service.container.in" "$rendered/$service-$idle.container"

say "Logging in to $registry..."
as_target podman login -u "$REGISTRY_USER" -p "$REGISTRY_TOKEN" "$registry"

say "Pulling $image..."
as_target podman pull "$image"

# Snapshot the supporting units, so only those whose definitions change get
# restarted. The live colour's file is never reinstalled.
declare -A before
supporting=()
for f in deploy/*.container; do
  unit=$(basename "$f" .container)
  supporting+=("$unit")
  before[$unit]=$(user_systemctl cat "$unit.service" 2>/dev/null || true)
done

say "Installing quadlet files to $quadlet_dir..."
as_target install -d -m 0755 "$quadlet_dir" "$unit_dir"
as_target install -m 0644 \
  deploy/*.container deploy/*.network deploy/*.volume deploy/*.sql \
  "$rendered/$service-$idle.container" \
  "$quadlet_dir/"
as_target install -m 0644 "deploy/$service-proxy.socket" "$unit_dir/"
user_systemctl daemon-reload

for unit in "${supporting[@]}"; do
  if [[ $(user_systemctl cat "$unit.service" 2>/dev/null || true) != "${before[$unit]}" ]]; then
    say "Restarting $unit..."
    user_systemctl restart "$unit.service" || err "$unit failed to restart"
  fi
done

say "Starting $service-$idle on port $idle_port..."
user_systemctl restart "$service-$idle.service"

max_attempts=36
delay=5
healthy=""
for ((attempt = 1; attempt <= max_attempts; attempt++)); do
  running=$(as_target podman inspect --format '{{.ImageName}}' "$service-$idle" 2>/dev/null || true)
  status=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$idle_port/healthz" || true)
  if [[ $running == "$image" && $status == 200 ]]; then
    healthy=yes
    break
  fi
  echo "Attempt $attempt/$max_attempts ($status), retrying in ${delay}s..."
  sleep "$delay"
done

if [[ -z $healthy ]]; then
  err "$service-$idle never served; ${live:-nothing} keeps the traffic"
  user_systemctl status "$service-$idle.service" || true
  user_systemctl stop "$service-$idle.service" || true
  exit 1
fi
ok "$service-$idle serves $image"

# The socket stays open while the proxy restarts, so new connections wait for
# it rather than fail. Connections already open through the old proxy close.
say "Switching traffic to $idle..."
render "$idle" "deploy/$service-proxy.service.in" "$rendered/$service-proxy.service"
as_target install -m 0644 "$rendered/$service-proxy.service" "$unit_dir/"
user_systemctl daemon-reload
user_systemctl enable "$service-proxy.socket" "$service-proxy.service"
# Before blue/green, $service.service published port 3000 itself.
if user_systemctl is-active --quiet "$service.service"; then
  user_systemctl stop "$service.service"
  user_systemctl reset-failed "$service.service" || true
fi
user_systemctl start "$service-proxy.socket"
user_systemctl restart "$service-proxy.service"

status=$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/healthz || true)
if [[ $status != 200 ]]; then
  err "Public port answered $status after switching to $idle"
  exit 1
fi
ok "Traffic on $idle"

if [[ -n $live ]]; then
  say "Stopping $service-$live..."
  sleep 5
  user_systemctl stop "$service-$live.service"
fi

say "Pruning unused images..."
as_target podman image prune -af
