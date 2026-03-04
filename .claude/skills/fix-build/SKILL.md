---
name: fix-build
description: Diagnose and fix CI build failures
allowed-tools: Bash(just ci-*, git diff*, git status, git log), Read, Grep, Glob
---

# Fix Build

Diagnose and fix CI build failures on Forgejo.

## Step 1: Check CI Status

!`just ci-status`

!`just ci-failures`

## Step 2: Analyze the Failure

Based on which job(s) failed, investigate:

### Check job failed

- **Cause**: Formatting or linting issues
- **Fix**: Run `just check` locally to see errors, then `just fmt` to fix formatting

### Deps job failed

- **Cause**: deps-lock.json is stale after deps.edn changes
- **Fix**: Run `just deps-lock` to regenerate, then commit

### Test job failed
- **Cause**: Test failures or missing CSS
- **Fix**: Run `just test` locally to reproduce. Check for:
  - Missing `resources/public/app.css` (run `just css`)
  - E2E test failures (check `target/browser-sessions/` for screenshots)
  - Database connection issues

### Build job failed

- **Cause**: Nix build issues, often stale deps hash
- **Fix**: Use `/sync-deps` skill if hash mismatch, or check Nix errors

### Deploy job failed

- **Cause**: Container registry or SSH issues
- **Fix**: Check secrets and SSH connectivity to compute

## Step 3: View Logs

Fetch logs for a specific job using `fj-ex` (requires `just ci-login` first):

```bash
just ci-logs <run-number> <job-index>
```

Note: job-index is **0-based** (first job is 0, second is 1, etc.)

Example: `just ci-logs 437 0` fetches logs for the first job of run 437.

The run URL is also available for browser viewing:

!`just ci-status | grep URL`

## Step 4: Fix and Verify

1. Make the necessary fixes locally
2. Run `just` to verify all checks pass
3. Commit and push

$ARGUMENTS
