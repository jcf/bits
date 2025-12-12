//! Authentication and authorization.
//!
//! # Authorization Levels
//!
//! Three extractors provide different authorization levels:
//!
//! - **`AuthSession`**: Full session control (login, logout, cache management)
//!   - Use when you need session lifecycle methods
//!   - Example: `auth.login_user()`, `auth.session.renew()`, `auth.cache_clear_user()`
//!
//! - **`Authenticated(User)`**: Requires authentication, allows unverified users
//!   - Use when you need to know who the user is but verification isn't required
//!   - Automatically returns 401 if not authenticated
//!
//! - **`Verified(User)`**: Requires email verification
//!   - Use when the endpoint requires a verified user
//!   - Automatically returns 403 if not verified
//!
//! ## Endpoints by Authorization Level
//!
//! - **Public**: No authentication required
//!   - `get_realm` - Get current realm (tenant/platform/demo)
//!   - `check_subdomain` - Check subdomain availability (public demo feature)
//!
//! - **Session Management**: Uses `AuthSession`
//!   - `auth` - Sign in
//!   - `join` - Sign up
//!   - `sign_out` - Sign out
//!   - `get_session` - Check if request is authenticated
//!   - `verify_email_code` - Verify email with code (needs cache management)
//!   - `resend_verification_code` - Resend verification code
//!
//! - **Verified**: Uses `Verified` extractor
//!   - `change_password` - Change user password

pub mod core;
pub mod password;
pub mod registration;
pub mod session;
pub mod verification;

// Re-export core types
pub use core::{AuthError, SessionState, User, UserState};
#[cfg(feature = "server")]
pub use core::{AuthSession, Authenticated, Verified};

// Re-export forms
pub use password::ChangePasswordForm;
pub use registration::JoinForm;
pub use session::{AuthForm, AuthResponse};
pub use verification::{ResendForm, ResendResponse, VerifyEmailForm};

// Re-export server functions
pub use password::change_password;
pub use registration::join;
pub use session::{auth, get_realm, get_session, sign_out};
pub use verification::{resend_verification_code, verify_email_code};
