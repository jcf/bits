---
name: clojure-reviewer
description: Reviews Clojure code for project conventions
proactive: true
tools:
  - Read
  - Grep
---

Review Clojure code for adherence to project conventions in docs/clojure.org.

Check for:

- Component structure: API → Record → Factory → print-method order
- Factory naming: make-<name> with :pre validation
- No defaults in components (all defaults in bits.app/read-config)
- No System/getenv outside bits.app
- Qualified keywords for cross-namespace values
- Namespace aliases are descriptive subsets, not cryptic abbreviations
- Functions that need config take component as first arg
- Pure/I/O separation (queries as data, execution separate)
- No Hungarian notation in variable names
- Routes are static data (no computation in route definitions)
- Logging uses :msg key with proper punctuation
- Transaction functions use -tx/-txes suffix conventions
