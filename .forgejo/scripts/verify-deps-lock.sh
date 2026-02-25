#!/usr/bin/env bash
set -eu

red=$(tput setaf 1)
green=$(tput setaf 2)
cyan=$(tput setaf 6)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}==>${reset} ${bold}$*${reset}"
}

err() {
  echo "${red}error:${reset} ${bold}$*${reset}" >&2
}

ok() {
  echo "${green}ok:${reset} $*"
}

say "Verifying deps-lock.json..."

if [[ ! -f deps-lock.json ]]; then
  err "deps-lock.json not found"
  echo ""
  echo "Generate it with:"
  echo ""
  echo "just deps-lock"
  echo ""
  exit 1
fi

cp deps-lock.json deps-lock.json.orig
just deps-lock

if ! diff -q deps-lock.json.orig deps-lock.json >/dev/null 2>&1; then
  err "deps-lock.json is out of date"
  echo ""
  echo "Diff (first 50 lines):"
  diff deps-lock.json.orig deps-lock.json | head -50 || true
  echo ""
  echo "Regenerate and commit with:"
  echo ""
  echo "just deps-lock"
  echo ""
  rm -f deps-lock.json.orig
  exit 1
fi

rm -f deps-lock.json.orig
ok "deps-lock.json is up to date"
