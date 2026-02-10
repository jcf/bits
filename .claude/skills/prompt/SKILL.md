---
name: prompt
description: Create a new prompt document
allowed-tools: Bash(just prompt *)
---

# Create Prompt

Create a new prompt document in `.claude/prompts/`.

## Usage

`/prompt <title>` - Create a prompt with the given title

## Example

`/prompt Add user authentication` creates:
`.claude/prompts/20260210123456-add-user-authentication.org`

!`just prompt "$ARGUMENTS"`
