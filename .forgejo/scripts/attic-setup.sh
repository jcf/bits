#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: ATTIC_TOKEN=<token> attic-setup.sh"
}

if [[ -z ${ATTIC_TOKEN:-} ]]; then
  usage
  exit 22 # EINVAL
fi

attic login invetica https://attic.lan.invetica.co.uk "$ATTIC_TOKEN"
attic use invetica:invetica
