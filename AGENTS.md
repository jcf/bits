# Bits Project Guidelines

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
