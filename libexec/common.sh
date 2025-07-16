#!/usr/bin/env bash
# Common functions for bin scripts

# Colors
BLUE=$(tput setaf 4)
GREEN=$(tput setaf 2)
RED=$(tput setaf 1)
YELLOW=$(tput setaf 3)
BOLD=$(tput bold)
RESET=$(tput sgr0)

# Output functions (all output to stderr)
info() {
    echo "${BLUE}${BOLD}==>${RESET} ${BOLD}$1${RESET}" >&2
}

success() {
    echo "${GREEN}${BOLD}==>${RESET} ${BOLD}$1${RESET}" >&2
}

error() {
    echo "${RED}${BOLD}==>${RESET} ${BOLD}$1${RESET}" >&2
}

warn() {
    echo "${YELLOW}${BOLD}==>${RESET} ${BOLD}$1${RESET}" >&2
}