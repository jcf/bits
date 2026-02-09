# Bits Project Guidelines

## Git

Do not include `Co-Authored-By` trailers in commit messages.

## Running Commands

Use `just` for all common tasks (not `bin/*` scripts):

- `just test` - Run all tests
- `just test :e2e` - Run E2E tests only
- `just lint` - Run linting
- `just fmt` - Format code
- `just check` - Run all quality checks

See `just --list` for all available commands.

## Clojure

**Claude Code Restriction**: Do not write new Clojure implementations. Claude may:

- Suggest code examples in conversation
- Suggest plans and architectural approaches
- Research supporting technologies (libraries, patterns)
- Explain existing code and answer questions
- Identify files that need modification
- Perform mechanical transformations (renames, deletes, moves)

But Claude must not use Edit or Write tools to write new Clojure logic, functions, or implementations. All new Clojure code must be written by the user.

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

### Docstrings

Don't add redundant docstrings that just describe what's obvious from the name.

```clojure
;; Bad: Redundant docstring
(defn redirect
  "Redirect to URL. Options can include :session for session data."
  [url]
  ...)

;; Good: No docstring needed - the name says it all
(defn redirect
  [url]
  ...)

;; Good: Docstring adds non-obvious information
(defn random-sid
  "160-bit (20 byte) secure random, URL-safe base64 encoded."
  []
  ...)
```

If you need to document parameter shapes or return values, use specs.

### Configuration Lives in bits.app

All configuration originates in `bits.app/read-config`. This is the single source
of truth. Defaults are defined there, not scattered across component namespaces.

**Components never define defaults.** They receive config and use it as-is.

```clojure
;; BAD: Defaults in component namespace
(def ^:private default-argon-config
  {:alg :argon2id :iterations 3 :memory (* 64 1024)})

(defrecord Keymaster [argon-config ...]
  component/Lifecycle
  (start [this]
    (let [config (or argon-config default-argon-config)]  ; <- NO!
      ...)))

;; GOOD: Defaults in bits.app, component just uses what it's given
;; In bits.app:
(defn read-config []
  {:keymaster {:argon-config {:alg         :argon2id
                              :iterations  3
                              :memory      (* 64 1024)
                              :parallelism 1}
               :idle-timeout-days 30}
   ...})

;; In bits.crypto:
(defrecord Keymaster [argon-config idle-timeout-days ...]
  component/Lifecycle
  (start [this]
    ;; Just use argon-config directly - no defaults, no merging
    (assoc this :dummy-hash (derive this ...))))
```

**Why this matters:**

- **One place to look** — All defaults are in `bits.app/read-config`
- **Easy to understand** — No hunting through namespaces for where a value comes from
- **Environment overrides work** — devenv/env vars override `bits.app` values
- **Validation in one place** — `bits.spec` validates the config from `bits.app`

### Component Structure

Every component namespace follows the same structure:

1. **API functions** — Functions that operate on the component (component as first arg)
2. **Record** — The component record implementing `component/Lifecycle`
3. **Factory** — `make-<component>` function with `:pre` validation
4. **Print method** — Simplified representation for REPL/logs

**Specs in `bits.spec`** to avoid cyclic dependencies:

- Component configuration specs (e.g., `:bits.crypto/config`)
- Specs for namespaces that may require `bits.spec` (e.g., morph action specs)

Use literal keywords (`:bits.keymaster/config`) since `bits.spec` can't require
component namespaces. Each namespace that needs specs in `bits.spec` gets its own
section with a comment explaining why.

```clojure
;; In bits.spec (literal keywords, no requires):
(s/def :bits.crypto/argon map?)
(s/def :bits.crypto/idle-timeout-days pos-int?)
(s/def :bits.crypto/config
  (s/keys :req-un [:bits.crypto/argon
                   :bits.crypto/idle-timeout-days]))

;; In bits.crypto:
(ns bits.crypto
  (:require
   [bits.cryptex :as cryptex]
   [bits.spec]
   [buddy.hashers :as hashers]
   [clojure.spec.alpha :as s]
   [com.stuartsierra.component :as component]))

;;; ----------------------------------------------------------------------------
;;; API

(defn derive
  [keymaster cryptex]
  (hashers/derive (cryptex/reveal cryptex) (:argon keymaster)))

(defn verify
  [_keymaster cryptex hash]
  (hashers/verify (cryptex/reveal cryptex) hash))

;;; ----------------------------------------------------------------------------
;;; Component

(defrecord Keymaster [argon dummy-hash idle-timeout-days]
  component/Lifecycle
  (start [this] ...)
  (stop [this] ...))

(defn make-keymaster
  [config]
  {:pre [(s/valid? ::config config)]}
  (map->Keymaster config))

(defmethod print-method Keymaster
  [keymaster ^java.io.Writer w]
  (.write w (format "#<Keymaster idle-timeout-days=%d>"
                    (:idle-timeout-days keymaster))))
```

**Key points:**

- **Factory is always `make-<name>`** — `make-keymaster`, `make-service`, `make-pool`
- **Factory takes config map** — Not destructured args with defaults
- **`:pre` validates with spec** — Catches config errors at system construction
- **Specs in `bits.spec`** — Avoids cyclic dependencies between namespaces
- **Print method hides internals** — Don't dump hashes, connections, etc.
- **API functions take component first** — Enables testing with mock components
- **No defaults in component** — All defaults live in `bits.app/read-config`

### Functions That Need Config Take a Component

When a function needs configuration, it takes the component as its first
argument. The component holds the config.

**Never add a configuration parameter to an existing function.** Configuration
implies state, state implies a component, and the component goes first.

```clojure
;; BAD: Adding config as an extra parameter
(defn derive
  [cryptex config]  ; <- NO! Don't tack on parameters
  (hashers/derive (cryptex/reveal cryptex) config))

;; GOOD: Component owns config and functions that use it
(defn derive
  [keymaster cryptex]
  (hashers/derive (cryptex/reveal cryptex) (:argon-config keymaster)))
```

**Rationale:**

- **Testability** — Pass a test component with fast/mock config
- **Single source of truth** — Config lives in the component
- **No parameter creep** — Components don't accumulate positional args
- **Clear ownership** — The component that owns the config owns the functions

### Routes Are Static Data

Route definitions are pure data. No computation, no function calls, no normalization.
Computation happens in `make-app` or component startup, not in route definitions.

```clojure
;; Good: Static data
(def routes
  [["/" {:get home-handler}]
   ["/action" {:post {:handler action-handler}}]])

;; Bad: Computation in route definition
(def routes
  [["/" {:get home-handler}]
   ["/action" {:post {:parameters {:form (build-schema actions)}  ; <- NO!
                      :handler (make-handler (normalize actions))}}]])  ; <- NO!
```

Normalization, schema building, and handler construction happen in `make-app` or
component startup - never at namespace load time.

### Variable Naming

Avoid Hungarian notation. Don't encode types in names when context is clear.

```clojure
;; Good: Plain names, context is clear
(let [action (get-in request [:parameters :form :action])]
  (get actions action))

;; Bad: Hungarian notation
(let [action-kw (get-in request [:parameters :form :action])]
  (get actions action-kw))
```

**Rationale:**

- **Redundant** — The code shows it's a keyword; the name doesn't need to
- **Noisy** — Suffixes clutter the code without adding information
- **Brittle** — If the type changes, the name lies

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

### Coercion at Boundaries

Use middleware coercion to convert external representations at system boundaries.
Keep internal code working with native types (keywords, not strings).

```clojure
;; Good: Coercion in middleware, handler receives keyword
["/action"
 {:post {:parameters {:form {:action :keyword}}
         :handler    (action-handler actions)}}]

(defn action-handler [actions]
  (fn [request]
    (let [action (get-in request [:parameters :form :action])]
      (get actions action))))

;; Bad: Manual string→keyword conversion in handler
(defn action-handler [actions]
  (fn [request]
    (let [action-str (get-in request [:params "action"])
          action     (keyword action-str)]
      (get actions action))))
```

**Rationale:**

- **Separation of concerns** — Parsing/coercion happens once at the boundary
- **Cleaner handlers** — Business logic works with native types
- **Declarative** — Parameter specs document the expected types

### HTTP Headers

Use **lowercase** for all header names and Ring utilities for access.

```clojure
;; Good: lowercase headers, Ring utilities
(response/get-header request "content-type")
{:headers {"content-type" "text/html"}}

;; Bad: Camel-Case headers, direct map access
(get-in request [:headers "Content-Type"])
{:headers {"Content-Type" "text/html"}}
```

- `ring.util.response/get-header` — case-insensitive read
- `ring.util.response/update-header` — case-insensitive update

**Rationale:**

- **Consistency** — Ring normalizes request headers to lowercase; match this for responses
- **Case-insensitive access** — Ring utilities handle case variations safely
- **HTTP spec compliance** — Header names are case-insensitive per RFC 7230

### Qualified Keywords as Domain Identifiers

Namespace-qualified keywords identify domain entities, not code locations. One entity
gets one keyword, used everywhere in the codebase.

```clojure
;; Good: One canonical name for the entity, used everywhere
;; The keymaster component owns :bits.crypto/keymaster
;; Every namespace that needs it uses the same keyword
(get request :bits.crypto/keymaster)
(assoc ctx :bits.crypto/keymaster km)

;; Bad: Different keywords per namespace for the same thing
;; Now you need specs for ::auth/keymaster, ::next/keymaster, ::app/keymaster
(get request ::keymaster)  ; in bits.auth → :bits.auth/keymaster
(get request ::keymaster)  ; in bits.next → :bits.next/keymaster

;; Good: User email is :user/email everywhere
{:user/email "alice@example.com"}

;; Bad: Same data, different names per context
{:auth/email "..."}   ; in auth namespace
{:form/email "..."}   ; in form namespace
{:db/email "..."}     ; in database namespace
```

**Rationale:**

- **Single source of truth** — One spec for `:user/email` validates it everywhere
- **Grep-ability** — Search for `:user/email` finds all uses; aliases fragment this
- **Domain modeling** — Keywords represent the domain, not code organization
- **RDF/Linked Data philosophy** — Identifiers are global; namespace is part of identity

**When to use `::` (auto-resolved keywords):**

- For keywords truly local to a namespace (internal implementation details)
- For keywords that represent "this namespace's contribution" to data

**When to use explicit namespaces:**

- For domain entities shared across namespaces
- For specs that should be reused
- For anything you'd want to grep for across the codebase

### Dev Namespace Conventions

Functions in `dev/` namespaces that are internal helpers should be private (`defn-`).
Keep I/O in the REPL comment block, not in functions.

```clojure
;; Good: Private helper builds pure data, I/O in comment block
(defn- user-txes
  [email password-hash]
  [{:db/id "user" :user/id (random-uuid) ...}
   {:email/address email :email/user "user"}])

(comment
  (datahike/transact! (:datahike system) (user-txes "dev@bits.page" hash)))

;; Bad: Function takes system map and does I/O
(defn create-user! [system email password]
  (let [hash (crypto/derive (:keymaster system) ...)]
    (datahike/transact! (:datahike system) ...)))
```

**Never pass the system map to a function.** Keep I/O at the call site.
