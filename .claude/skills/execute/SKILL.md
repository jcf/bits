---
name: execute
description: Select and execute a prompt from .claude/prompts/
allowed-tools: Glob, Read, Bash, Edit, Write, AskUserQuestion
---

# Execute Prompt

Execute a prompt from `.claude/prompts/`.

## Usage

- `/execute` - List prompts, select interactively
- `/execute <slug>` - Execute specific prompt by slug (filename without extension)
- `/execute --all` - Include completed prompts in selection

## Process

### Step 1: Find Prompts

List available prompts:

!`ls -1 .claude/prompts/*.org 2>/dev/null | while read f; do status=$(grep -m1 '^#+status:' "$f" | sed 's/#+status: *//'); title=$(grep -m1 '^#+title:' "$f" | sed 's/#+title: *//'); basename=$(basename "$f" .org); echo "[$status] $title ($basename)"; done`

### Step 2: Select Prompt

If `$ARGUMENTS` contains a slug, use it directly.

Otherwise, use AskUserQuestion to let the user select from the list above.
Skip prompts with status `done` unless `--all` was specified.

### Step 3: Read and Parse

Read the selected prompt file. Note the:
- Title (`#+title:`)
- Status (`#+status:` - todo, doing, done)
- Goals and success criteria

### Step 4: Execute

Follow this process:

1. **Understand** - Read the prompt thoroughly
2. **Improve** - Consider better approaches, edge cases
3. **Derisk** - What could go wrong? Address assumptions
4. **Plan** - Formulate concrete steps with checkpoints
5. **Present** - Show the plan before executing
6. **Implement** - Make changes systematically
7. **Verify** - Run tests to confirm success

### Step 5: Update Status

As work progresses, update the prompt's `#+status:` field:
- `todo` - Not started
- `doing` - In progress
- `done` - Completed

## Important

- Follow all guidelines in CLAUDE.md
- Run tests after changes
- Keep commits focused
- Update the prompt doc with important decisions and status changes
