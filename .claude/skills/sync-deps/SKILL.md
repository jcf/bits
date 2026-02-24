---
name: sync-deps
description: Synchronise Nix deps hash after changing deps.edn
---

# Sync Dependencies Hash

Update the fixed-output derivation hash in `pkgs/bits-uberjar/default.nix` after
changing `deps.edn`.

## Background

The uberjar build uses a fixed-output derivation (FOD) to cache Maven/Clojure
dependencies. When `deps.edn` changes, the hash becomes stale and builds fail
with a hash mismatch error containing the correct hash.

## Process

1. Read `pkgs/bits-uberjar/default.nix`

2. Edit `depsHash` to an invalid value to force recomputation:

   ```nix
   depsHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
   ```

3. Clear devenv cache: `rm -f .devenv/nix-eval-cache.db`

4. Run: `devenv build outputs.bits-uberjar`

   The build fails with a hash mismatch. Find the `got:` line containing the
   correct hash (format: `sha256-...=`).

5. Edit `depsHash` with the correct hash from the error output.

6. Verify: `devenv build outputs.bits-uberjar`

7. If verification fails, restore: `git checkout pkgs/bits-uberjar/default.nix`

$ARGUMENTS
