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

## Dioxus

### Application Structure

Organize Dioxus applications with clear separation between pages, components, and app configuration:

```
src/
├── app.rs              # AppState, Route enum, root App component
├── components/         # Reusable UI components
│   ├── mod.rs         # Module declarations + re-exports
│   ├── button.rs      # Button component
│   ├── header.rs      # Header component
│   └── ...
├── pages/             # Route-level page components
│   ├── mod.rs        # Module declarations + re-exports
│   ├── layout.rs     # Shared layout with error boundaries
│   ├── home.rs       # Home page
│   ├── auth.rs       # Auth page
│   └── ...
├── lib.rs            # Module declarations + public API
└── ...
```

**Organization principles:**

1. **One component per file** - Each component gets its own dedicated file
2. **Pages vs components** - Pages correspond to routes, components are reusable UI elements
3. **Explicit module declarations** - Use `mod.rs` to declare modules and re-export public items
4. **Centralized routing** - Define routes in `app.rs` alongside the root `App` component
5. **Server-only modules** - Use `#[cfg(feature = "server")]` for server-side code

**Module pattern (`mod.rs`):**

```rust
// src/components/mod.rs
pub mod button;
pub mod header;
pub mod avatar;

pub use button::{Button, ButtonVariant, ButtonSize};
pub use header::Header;
pub use avatar::Avatar;
```

**App module (`app.rs`):**

```rust
use crate::pages::{Auth, Home, Join, Layout};

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub config: std::sync::Arc<Config>,
    pub db: sqlx::PgPool,
    // ...
}

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/auth")]
    Auth {},
    // ...
}

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
```

**Public API (`lib.rs`):**

```rust
// Module declarations
pub mod app;
pub mod components;
pub mod pages;

#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod server;

// Re-exports for public API
pub use app::{App, AppState, Route};
pub use components::{Button, Header};
```

**Rationale:**

- **Clear boundaries** - Pages handle routing, components handle UI, app handles configuration
- **Discoverability** - One component per file makes it easy to find code
- **Explicit exports** - Re-exports in `mod.rs` and `lib.rs` define the public API
- **Feature gating** - Server code only compiles when needed, reducing WASM bundle size
- **Layout component** - Shared layout with error boundaries provides consistent structure

### Server Functions

**CRITICAL: Use `#[post]`/`#[get]` macros, NOT `#[server]`.**

The `#[server]` macro generates generic endpoints. Use HTTP method-specific macros instead for explicit routing and better type safety.

**Dioxus server functions generate both client stubs and server implementations.**

The `#[post]` and `#[get]` macros create TWO functions:
1. **Client stub** - Called from browser, serializes parameters and makes HTTP request
2. **Server handler** - Runs on server, deserializes parameters and executes logic

**Server-only dependencies (database, auth session, config) cannot be in the function signature** because the client stub must compile without them. They're extracted from request context instead.

```rust
// CORRECT: Server-only extractors in macro, serializable params in signature
#[post("/api/sessions", auth: AuthSession, state: Extension<crate::AppState>)]
pub async fn auth(form: dioxus::fullstack::Form<AuthForm>) -> Result<User, AuthError> {
    // auth and state injected by macro on server side only
    let user = load_login_data(&state.db, &form.0.email).await?;
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| AuthError::InvalidCredentials)?;
    auth.session.renew();
    auth.login_user(user_id);
}

// Client can call this:
// let user = auth(AuthForm { email, password }).await?;
// Client stub only needs to serialize AuthForm, not AppState/AuthSession

// WRONG: Don't use #[server]
#[server]  // ❌ Use #[post] or #[get] instead
pub async fn auth(form: AuthForm) -> Result<User, AuthError> { }

// WRONG: Server-only types in signature break client compilation
#[post("/api/sessions")]
pub async fn auth(
    auth: AuthSession,  // ❌ Client can't construct this
    state: Extension<crate::AppState>,  // ❌ Client has no AppState
    form: dioxus::fullstack::Form<AuthForm>
) -> Result<User, AuthError> { }
```

**In the macro (server-only extraction):**
- `auth: AuthSession` - Extracted from request extensions
- `state: Extension<crate::AppState>` - Extracted from request extensions
- Any type that only exists on the server

**In the signature (client can provide):**
- Request body/form data that serializes over the wire
- Primitive types, strings, serializable structs
- `dioxus::fullstack::Form<T>` where `T: Serialize + Deserialize`

This is how Dioxus fullstack achieves isomorphic functions - client calls with serializable args, server extracts context automatically.

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
