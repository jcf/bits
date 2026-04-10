#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: cleanup-images.sh <registry> <owner> <package> [keep]"
  echo >&2 ""
  echo >&2 "Delete old container images from Forgejo registry, keeping the most"
  echo >&2 "recent versions."
  echo >&2 ""
  echo >&2 "Arguments:"
  echo >&2 "  registry  Forgejo instance (e.g., code.invetica.team)"
  echo >&2 "  owner     Package owner (e.g., jcf)"
  echo >&2 "  package   Package name (e.g., bits)"
  echo >&2 "  keep      Number of versions to keep (default: 1)"
  echo >&2 ""
  echo >&2 "Environment:"
  echo >&2 "  REGISTRY_TOKEN  API token with package:write scope"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  cleanup-images.sh code.invetica.team jcf bits 1"
}

if [[ $# -lt 3 || $# -gt 4 ]]; then
  usage
  exit 22
fi

registry=$1
owner=$2
package=$3
keep=${4:-1}

cyan=$(tput setaf 6)
green=$(tput setaf 2)
yellow=$(tput setaf 3)
bold=$(tput bold)
reset=$(tput sgr0)

say() { echo >&2 "${cyan}→${reset} ${bold}$*${reset}"; }
ok() { echo >&2 "${green}ok:${reset} $*"; }
warn() { echo >&2 "${yellow}skip:${reset} $*"; }

api="https://$registry/api/v1"
auth="Authorization: token $REGISTRY_TOKEN"

say "Listing $package versions..."

# Get all versions, sorted by creation date (newest first)
versions=$(curl -sf -H "$auth" "$api/packages/$owner?type=container" |
  jq -r ".[] | select(.name == \"$package\") | .version" |
  sort -rV)

count=$(echo "$versions" | grep -c . || echo 0)
say "Found $count versions"

if [[ $count -le $keep ]]; then
  ok "Nothing to delete (keeping $keep)"
  exit 0
fi

# Skip the first $keep versions, delete the rest
to_delete=$(echo "$versions" | tail -n +$((keep + 1)))

for version in $to_delete; do
  say "Deleting $package:$version..."
  if curl -sf -X DELETE -H "$auth" "$api/packages/$owner/container/$package/$version"; then
    ok "Deleted"
  else
    warn "Failed to delete $version"
  fi
done

say "Cleanup complete"
