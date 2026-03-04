---
name: fix-build
description: Diagnose and fix CI build failures
allowed-tools: Bash(just ci-*, git diff*, git status, git log, gh run download), Read, Grep, Glob, ToolSearch, Task, mcp__clojure-mcp__*
---

# Fix Build

Diagnose and fix CI build failures on Forgejo.

## Step 1: Check CI Status

!`just ci-status`

!`just ci-failures`

## Step 2: Fetch Logs for Failed Jobs

From the ci-status output above, note the run number (e.g., "Run #437").

Get the job list to find indices:

```bash
just ci-jobs <run-number>
```

Fetch logs for a specific job (job-index is **0-based**):

```bash
just ci-logs <run-number> <job-index>
```

Test job is usually index 2.

## Step 3: Analyze the Failure

Based on which job(s) failed, investigate:

### Check job failed

- **Cause**: Formatting or linting issues
- **Fix**: Run `just check` locally to see errors, then `just fmt` to fix formatting

### Deps job failed

- **Cause**: deps-lock.json is stale after deps.edn changes
- **Fix**: Run `just deps-lock` to regenerate, then commit

### Test job failed

**Common patterns in test logs:**

| Pattern                     | Likely cause                                     |
| --------------------------- | ------------------------------------------------ |
| `Timeout (7):`              | E2E test timed out waiting for element/condition |
| `Wait until ... is visible` | UI element didn't appear after action            |
| `Failed: [[...]]`           | Generative test with shrunk failing case         |
| `aria-busy` stuck           | Form validation/morph not completing             |

**E2E timeout failures:**

- Check if the form submission flow is broken
- Look at the failing test line to understand what it's waiting for
- Browser screenshots are uploaded as artifacts (see Step 4)

**Generative test failures:**

- The log shows the shrunk failing sequence, e.g.: `Failed: [[[:type :number "aAA"] ...]]`
- Use the REPL to reproduce the exact sequence against a running system
- Check form invariants: aria-invalid consistency, describedby refs exist

**Quick local reproduction:**

```bash
just test :e2e           # Run only E2E tests
just test :generative    # Run only generative tests
```

### Build job failed

- **Cause**: Nix build issues, often stale deps hash
- **Fix**: Use `/sync-deps` skill if hash mismatch, or check Nix errors

### Deploy job failed

- **Cause**: Container registry or SSH issues
- **Fix**: Check secrets and SSH connectivity to compute

## Step 4: Download Browser Artifacts

CI uploads browser screenshots on E2E failures. Download and examine:

```bash
# List artifacts for the run
gh api repos/jcf/bits/actions/runs/<run-id>/artifacts

# Download artifact (artifact ID shown in test logs)
curl -L -o screenshots.zip "<artifact-url>"
unzip screenshots.zip -d target/ci-screenshots
```

The artifact download URL appears in test logs as:
`Artifact download URL: https://code.invetica.team/jcf/bits/actions/runs/.../artifacts/...`

## Step 5: REPL-Based Debugging

For complex test failures, use the REPL (requires nREPL on port 9999):

```clojure
;; Run specific failing test
(require '[clojure.test :refer [test-var]])
(test-var #'bits.form-test/form-reset)

;; Reproduce generative test failure with exact sequence
(require '[bits.form.gen-test :as gen-test])
(require '[bits.test.browser :as browser])
;; Execute the shrunk failing actions manually
```

## Step 6: Fix and Verify

1. Make the necessary fixes locally
2. Run `just` to verify all checks pass
3. Commit and push

$ARGUMENTS
