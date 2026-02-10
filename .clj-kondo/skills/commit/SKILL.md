---
name: commit
description: Create atomic commits from session changes only
allowed-tools: Bash(git status, git diff*, git add, git commit, git log)
---

# Commit Session Changes

Create atomic commits for **only the changes we made together** in this session.

## Step 1: Identify Session Changes

Review your tool use history from this conversation. List the files you edited
using Edit or Write tools. These are "our changes."

## Step 2: Cross-Reference with Git

!`git status --short`

!`git diff --stat`

Only consider files that:

- You edited during this session (from step 1)
- Actually have uncommitted changes (from git status)

Ignore any pre-existing uncommitted changes that weren't part of our work.

## Step 3: Propose Atomic Commits

Group related changes into logical commits. For each proposed commit, show:

Commit 1:

- path/to/file1.clj
- path/to/file2.clj

Commit 2:

- path/to/other.clj

## Step 4: Wait for Approval

Present the plan and ask: "Ready to create these commits?"

**Do not execute git commands until the user approves.**

## Commit Style

- Imperative mood, lowercase, no trailing period
- No Co-Authored-By trailers
- Match style of recent commits:

!`git log --oneline -5`

$ARGUMENTS
