#!/usr/bin/env bash
set -eu

usage() {
  echo >&2 "Usage: wait-postgres.sh <host> <port> <user> [timeout]"
  echo >&2 ""
  echo >&2 "Example:"
  echo >&2 "  wait-postgres.sh postgres 5432 bits"
  echo >&2 "  wait-postgres.sh localhost 5432 bits 60"
}

if [[ $# -lt 3 ]]; then
  usage
  exit 22
fi

host=$1
port=$2
user=$3
timeout=${4:-30}

cyan=$(tput setaf 6)
green=$(tput setaf 2)
red=$(tput setaf 1)
bold=$(tput bold)
reset=$(tput sgr0)

say() {
  echo "${cyan}==>${reset} ${bold}$*${reset}"
}

ok() {
  echo "${green}ok:${reset} $*"
}

err() {
  echo "${red}error:${reset} ${bold}$*${reset}" >&2
}

say "Waiting for PostgreSQL at ${host}:${port}..."

for ((i = 1; i <= timeout; i++)); do
  if pg_isready -h "$host" -p "$port" -U "$user" >/dev/null 2>&1; then
    ok "PostgreSQL is ready"
    exit 0
  fi
  sleep 1
done

err "PostgreSQL failed to start within ${timeout}s"
exit 1
