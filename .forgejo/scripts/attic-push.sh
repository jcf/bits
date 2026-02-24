#!/usr/bin/env bash
set -eu

# Push gcroots and their closures to Attic
# This ensures future CI runs can fetch from Attic instead of cache.nixos.org
for path in .devenv/gc/*; do
  if [[ -L $path ]]; then
    # Resolve symlink and push the actual store path with its closure
    store_path=$(readlink -f "$path")
    attic push invetica:invetica "$store_path" || true
  fi
done
