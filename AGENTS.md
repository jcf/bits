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

**ABSOLUTE RULES - NEVER VIOLATE:**

1. **NEVER use naked booleans as function parameters** - Booleans as parameters are completely banned. Call sites like `foo(true)` are unreadable. Use enums instead.

```rust
// BANNED: Naked boolean parameter
pub fn csp_header(development: bool) -> String { /* ... */ }
csp_header(true)  // What does true mean? Unreadable!

// REQUIRED: Use enum instead
pub enum CspMode {
    Strict,
    Development,
}
pub fn csp_header(mode: CspMode) -> String { /* ... */ }
csp_header(CspMode::Development)  // Clear intent!
```

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

### SQL

Use lowercase for all SQL keywords and identifiers. Uppercase SQL is harder to read and visually jarring.

```rust
// Good: Lowercase SQL
sqlx::query_as::<_, User>(
    "select id, email from users where id = $1"
)

// Bad: Uppercase SQL
sqlx::query_as::<_, User>(
    "SELECT id, email FROM users WHERE id = $1"
)
```

### Configuration and Database Pools

Use a single `Config` with serde defaults for optional fields. Use `AppState::new()` to create database pools - don't manually create them.

```rust
// Good: Use AppState for pool creation
let config = bits_app::Config::from_env()?;
let state = bits_app::AppState::new(config).await?;
// Now use state.db for all database operations

// Bad: Manual pool creation
let config = bits_app::Config::from_env()?;
let pool = PgPoolOptions::new()
    .max_connections(config.max_database_connections)
    .connect(config.database_url.as_ref())
    .await?;

// Config with serde defaults (in config.rs):
fn default_max_database_connections() -> u32 { 5 }

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_max_database_connections")]
    pub max_database_connections: u32,
    // ...
}
```

**Rationale:**

- **One config system** - Don't create config variants for different use cases
- **Defaults in serde** - Allow fields to be optional in environment
- **AppState centralizes pool creation** - Reuse logic, don't duplicate
- **Builder pattern for overrides** - Use `.with_database_url()` etc. when needed (e.g., tests)

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

### Session Security

Follow OWASP guidelines for session management:

**Session rotation:**
- Rotate session IDs on authentication state changes (login, logout)
- Prevents session fixation attacks
- Use `auth.session.renew()` after successful authentication

**Cookie security:**
- `Secure`: Transmit only over HTTPS
- `HttpOnly`: Prevent JavaScript access
- `SameSite=Strict`: CSRF protection

**Password changes:**
- Invalidate ALL sessions when password changes (including current session)
- Forces re-authentication with new credentials
- If attacker compromised password, they're immediately logged out everywhere

```rust
// Good: Rotate session on login
Argon2::default()
    .verify_password(password.as_bytes(), &hash)
    .map_err(|_| AuthError::InvalidCredentials)?;
auth.session.renew();  // Prevent session fixation
auth.login_user(user_id);

// Good: Configure secure cookies
SessionConfig::default()
    .with_secure(true)
    .with_http_only(true)
    .with_cookie_same_site(SameSite::Strict)

// Good: Invalidate all sessions on password change
update_password_hash(&db, user_id, new_hash).await?;
invalidate_all_sessions(&db, user_id).await?;
auth.logout_user();  // Force re-authentication
```
