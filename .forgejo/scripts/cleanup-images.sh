#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: cleanup-images.sh <registry> <owner> <package> [keep]"
  echo >&2 ""
  echo >&2 "Delete old container images from Forgejo registry, keeping the most"
  echo >&2 "recent deploys."
  echo >&2 ""
  echo >&2 "Arguments:"
  echo >&2 "  registry  Forgejo instance (e.g., code.invetica.team)"
  echo >&2 "  owner     Package owner (e.g., jcf)"
  echo >&2 "  package   Package name (e.g., bits)"
  echo >&2 "  keep      Number of deploys to keep (default: 1)"
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

# Every tagged version with its creation time, across all pages. Untagged
# sha256 manifests are left alone, since a kept index may still point at
# them.
versions=$(
  page=1
  while :; do
    batch=$(curl -sf -H "$auth" "$api/packages/$owner?type=container&q=$package&limit=50&page=$page")
    [[ $(jq length <<<"$batch") -eq 0 ]] && break
    jq -r --arg p "$package" \
      '.[] | select(.name == $p and (.version | startswith("sha256:") | not)) | "\(.created_at) \(.version)"' \
      <<<"$batch"
    page=$((page + 1))
  done
)

# One deploy pushes <tag>-amd64 and the <tag> index, so count deploys by
# their base tag, newest first by creation time.
deploys=$(sort -r <<<"$versions" | cut -d' ' -f2 | sed -E 's/-(amd64|arm64)$//' | grep -vx latest | uniq)

count=$(grep -c . <<<"$deploys" || true)
say "Found $count deploys"

if [[ $count -le $keep ]]; then
  ok "Nothing to delete (keeping $keep)"
  exit 0
fi

kept=$(head -n "$keep" <<<"$deploys")
to_delete=$(cut -d' ' -f2 <<<"$versions" | grep -vx latest | while read -r version; do
  grep -qx "$(sed -E 's/-(amd64|arm64)$//' <<<"$version")" <<<"$kept" || echo "$version"
done)

for version in $to_delete; do
  say "Deleting $package:$version..."
  if curl -sf -X DELETE -H "$auth" "$api/packages/$owner/container/$package/$version"; then
    ok "Deleted"
  else
    warn "Failed to delete $version"
  fi
done

say "Cleanup complete"
