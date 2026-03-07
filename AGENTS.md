# Bits Project Guidelines

## Quick Reference

| Task       | Command          |
| ---------- | ---------------- |
| All checks | `just`           |
| Tests      | `just test`      |
| E2E only   | `just test :e2e` |
| Format     | `just fmt`       |
| Lint       | `just lint`      |

## Skills & Documentation

Use these skills for task-specific guidance:

| Skill        | When to use                 |
| ------------ | --------------------------- |
| `/test`      | Writing/debugging tests     |
| `/form`      | Form UI components          |
| `/component` | Creating Clojure components |
| `/fix-build` | CI failures                 |
| `/commit`    | Atomic commits              |

Reference docs (read when working in these areas):

- `docs/clojure.org` — Clojure conventions
- `docs/testing.org` — Testing patterns
- `docs/forms.org` — Form interaction design

## MCP Servers

| Server      | Purpose                      |
| ----------- | ---------------------------- |
| clojure-mcp | REPL integration (port 9999) |
| devenv      | Packages and scripts         |
| linear      | Issue tracking               |

If clojure-mcp fails to connect, ask the user to start the REPL (`devenv up`).

### clojure-mcp Tools

| Tool                  | Purpose                  |
| --------------------- | ------------------------ |
| `clojure_eval`        | Evaluate Clojure in REPL |
| `clojure_edit`        | Structure-aware editing  |
| `deps_list/grep/read` | Explore dependencies     |

## REPL-Driven Development

With REPL access, follow **verify-then-commit**:

1. Prototype in REPL
2. Verify behavior (run tests, check edge cases)
3. Iterate until correct
4. Write to file

### Testing via REPL

```clojure
(require '[clojure.test :refer [run-tests test-var]])
(run-tests 'bits.foo-test)           ; namespace
(test-var #'bits.foo-test/my-test)   ; single test
```

Use REPL for iteration; use `just test` before commits.

## Git

- **Forgejo** at `code.invetica.team` (not GitHub—`gh` CLI won't work)
- No `Co-Authored-By` trailers in commit messages

## Critical Constraints

These apply at all times—violations are never acceptable.

### BANNED

- **Hiccup class shorthand** — Never `[:div.foo]`, always `[:div {:class "foo"}]`
- **Arbitrary sleeps in tests** — Use condition-based waits
- **`System/getenv` outside bits.app** — All config in `bits.app/read-config`
- **Flash messages/toasts** — Forms use physical feedback (shake, color)
- **Glob imports in Rust** — Always explicit imports

### REQUIRED

- **Qualified keywords** for values crossing namespace boundaries
- **Component first arg** for functions needing config
- **Lowercase HTTP headers** with Ring utilities
- **`make-<name>` factory** with `:pre` spec validation

## Rust

### Imports

Avoid glob imports (`use module::*`). Always explicit imports or module paths.

### Parameters

1. I/O components first (AppState, Database, etc.)
2. Natural ordering for remaining (scheme, then host)
3. Use structs when >2-3 parameters

## OpenTelemetry

```clojure
(span/with-span! {:name       ::my-operation
                  :attributes {"tenant_id" (str id)}}
  ...)
```

- Span names: auto-resolved keywords (`::name`)
- Attributes: string keys, dot-namespaced snake_case

## Org-mode

Minimal whitespace. No blank line after heading before content.

```org
* Heading
** Subheading
Content here.

** Another heading
More content.
```
