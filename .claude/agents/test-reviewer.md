---
name: test-reviewer
description: Reviews tests for project conventions
proactive: true
tools:
  - Read
  - Grep
---

Review tests for adherence to conventions in docs/testing.org.

Check for:

- No arbitrary Thread/sleep (use condition-based waits)
- No println in test helpers (return data, assert with is)
- Tests check semantic state via ARIA, not visual presentation
- Test system customized with assoc-in, not optional params
- Proper metadata for test filtering (:e2e, :generative)
- Test helpers return data; assertions in deftest using is
- Systematic solutions over quick fixes
