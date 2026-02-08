# Bits Project Guidelines

## Git

Do not include `Co-Authored-By` trailers in commit messages.

## Rust

### Imports

Avoid glob imports (`use module::*`) - they make it hard to trace where items
come from. Always use explicit imports or module paths.

```rust
// Preferred: Explicit imports
use bits_e2e::server::spawn_colo;
use bits_e2e::fixtures::create_tenant;

// Or: Module-qualified usage
use bits_e2e::{server, fixtures};

fn test() {
    let srv = server::spawn_colo();
    fixtures::create_tenant("name");
}

// Avoid: Glob imports
use bits_e2e::fixtures::*;  // Hard to trace where create_tenant comes from
```

### Function Parameters

**Parameter ordering rules:**

1. **I/O components first** - Any stateful I/O component (AppState, Database, FileHandle, etc.) must be the first parameter. This enables testing via mocking/stubbing.
2. **Natural ordering for remaining parameters** - Match real-world structure (e.g., URLs: scheme then host).
3. **Use structs when >2-3 parameters** - Order doesn't matter in structs. Don't force callers to remember arbitrary positional order.

```rust
// Good: I/O component first, then natural URL ordering (scheme, host)
pub async fn resolve_realm(
    state: &AppState,
    scheme: Scheme,
    host: &str
) -> Realm {
    let normalized = normalize_host(scheme, host);
    load_tenant_by_domain(&state.db, &normalized).await
}

// Bad: Separate I/O components instead of unified state
pub async fn resolve_realm(
    host: &str,
    config: &Config,
    db: &PgPool
) -> Realm { /* ... */ }

// Bad: Too many positional parameters (hard to remember order)
pub async fn create_tenant(
    state: &AppState,
    slug: &str,
    email: &str,
    name: &str,
    plan: &str,
    trial_days: u32
) -> Result<Tenant> { /* ... */ }

// Good: Use struct for many parameters
pub struct CreateTenantParams {
    pub slug: String,
    pub email: String,
    pub name: String,
    pub plan: String,
    pub trial_days: u32,
}

pub async fn create_tenant(
    state: &AppState,
    params: CreateTenantParams
) -> Result<Tenant> { /* ... */ }
```

**Rationale:**

- **I/O first enables testing** - Mock/stub the I/O component to test logic in isolation
- **Single state object is easier** - Thread one parameter instead of many
- **Natural ordering is memorable** - `https://example.com` → scheme, host
- **Structs for complex cases** - Caller doesn't need to remember arbitrary positional order

### Testing

Organize integration test utilities in `src/lib.rs` with public modules:

```rust
// crates/bits-e2e/src/lib.rs
pub mod server;
pub mod fixtures;
pub mod request;

// tests/integration.rs
use bits_e2e::{server, fixtures};

#[tokio::test]
async fn test_behavior() {
    let srv = server::spawn_colo().await;
    let tenant = fixtures::create_tenant("jcf").await;
    // Test logic
}
```

## Clojure

### Namespace Aliases

Use descriptive aliases that are subsets of the full namespace. Avoid cryptic
abbreviations that require mental lookup.

```clojure
;; Good: Descriptive subset of namespace
[reitit.coercion.malli :as coercion.malli]
[reitit.ring.coercion :as ring.coercion]
[clojure.core.async :as a]

;; Bad: Cryptic abbreviations
[reitit.coercion.malli :as rcm]
[reitit.ring.coercion :as rrc]
```

**Rationale:**

- **Readability** — Code reads naturally without mental lookup
- **Discoverability** — New readers can trace back to the source namespace
- **Consistency** — Aliases match namespace structure

### Separation of Pure and I/O

Keep pure data separate from functions that perform I/O. Queries should be
defined as pure data; execution should happen in separate functions.

```clojure
;;; ----------------------------------------------------------------------------
;;; Queries (pure data)

(def user-by-email-query
  '[:find (pull ?u [:user/id :user/password-hash]) .
    :in $ ?email
    :where
    [?e :email/address ?email]
    [?e :email/user ?u]])

;;; ----------------------------------------------------------------------------
;;; I/O (functions that execute queries)

(defn find-by-email
  [database email]
  (datahike/q user-by-email-query (datahike/db database) email))
```

**Rationale:**

- **Testability** - Pure query data can be inspected and validated without I/O
- **Reusability** - Same query can be used with different execution contexts
- **Clarity** - Clear separation between "what" (query) and "how" (execution)

**Notes:**

- Use `.` after find pattern for unique attribute lookups — returns single
  result directly (or nil) instead of a set
- Use `(pull ?e [...])` to get maps with exact keys — no post-processing
- Keep I/O functions minimal — just execute the query
