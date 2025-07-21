use bits_identity::{IdentityService, generate_username};
use bits_core::Component;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{info, error};

/// Shared application state
#[derive(Clone)]
struct AppState {
    identity_service: Arc<RwLock<IdentityService>>,
}


/// API response for identity info
#[derive(Serialize)]
struct IdentityResponse {
    did: String,
    username: String,
    public_keys: PublicKeysInfo,
    created_at: String,
}

#[derive(Serialize)]
struct PublicKeysInfo {
    master: String,
    signing: String,
    authentication: String,
}

/// Sign message request
#[derive(Deserialize)]
struct SignRequest {
    message: String,
}

/// Sign message response
#[derive(Serialize)]
struct SignResponse {
    signature: String,
    signer_did: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    
    info!("Starting Bits Identity Service");
    
    // Create identity service
    let mut identity_service = IdentityService::new("./data").await?;
    identity_service.start().await?;
    
    // Get initial identity info for logging
    let identity = identity_service.current().await?;
    let username = generate_username(&identity.did);
    info!("Identity loaded: {} ({})", username, identity.did);
    
    // Create shared state
    let state = AppState {
        identity_service: Arc::new(RwLock::new(identity_service)),
    };
    
    // Build router
    let app = Router::new()
        // Web routes
        .route("/", get(home_handler))
        .route("/profile/:username", get(profile_handler))
        
        // API routes
        .route("/api/identity", get(get_identity))
        .route("/api/identity/sign", post(sign_message))
        .route("/api/identity/backup", post(create_backup))
        
        // Add state and middleware
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Web server listening on http://{}", addr);
    info!("Visit http://{}/profile/{}", addr, username);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Web handlers

async fn home_handler(State(state): State<AppState>) -> impl IntoResponse {
    let service = state.identity_service.read().await;
    match service.current().await {
        Ok(identity) => {
            let username = generate_username(&identity.did);
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Bits Identity Service</title>
    <style>
        body {{ font-family: system-ui; max-width: 800px; margin: 40px auto; padding: 0 20px; }}
        .container {{ background: #f5f5f5; padding: 30px; border-radius: 10px; }}
        h1 {{ color: #333; }}
        .info {{ background: white; padding: 20px; border-radius: 5px; margin: 20px 0; }}
        .username {{ font-size: 24px; font-weight: bold; color: #5e72e4; }}
        .did {{ font-family: monospace; font-size: 14px; color: #666; word-break: break-all; }}
        a {{ color: #5e72e4; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 Bits Identity Service</h1>
        <div class="info">
            <p>Your identity:</p>
            <p class="username">{}</p>
            <p class="did">{}</p>
            <p><a href="/profile/{}">View your profile →</a></p>
        </div>
        <div class="info">
            <h3>API Endpoints</h3>
            <ul>
                <li><code>GET /api/identity</code> - Get identity information</li>
                <li><code>POST /api/identity/sign</code> - Sign a message</li>
                <li><code>POST /api/identity/backup</code> - Create encrypted backup</li>
            </ul>
        </div>
    </div>
</body>
</html>"#,
                username, identity.did, username
            );
            Html(html)
        }
        Err(e) => {
            error!("Failed to get identity: {}", e);
            Html("<h1>Error loading identity</h1>".to_string())
        }
    }
}

async fn profile_handler(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let service = state.identity_service.read().await;
    match service.current().await {
        Ok(identity) => {
            let current_username = generate_username(&identity.did);
            
            if username != current_username {
                return (StatusCode::NOT_FOUND, Html("<h1>Profile not found</h1>".to_string()));
            }
            
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - Bits Profile</title>
    <style>
        body {{ font-family: system-ui; max-width: 800px; margin: 40px auto; padding: 0 20px; }}
        .profile {{ background: #f5f5f5; padding: 30px; border-radius: 10px; }}
        h1 {{ color: #333; margin-bottom: 5px; }}
        .did {{ font-family: monospace; font-size: 14px; color: #666; word-break: break-all; }}
        .info-box {{ background: white; padding: 20px; border-radius: 5px; margin: 20px 0; }}
        .key {{ font-family: monospace; font-size: 12px; color: #888; }}
        .created {{ color: #666; font-size: 14px; }}
    </style>
</head>
<body>
    <div class="profile">
        <h1>@{}</h1>
        <p class="did">{}</p>
        <p class="created">Member since {}</p>
        
        <div class="info-box">
            <h3>Public Keys</h3>
            <p><strong>Master:</strong> <span class="key">{}</span></p>
            <p><strong>Signing:</strong> <span class="key">{}</span></p>
            <p><strong>Authentication:</strong> <span class="key">{}</span></p>
        </div>
        
        <div class="info-box">
            <h3>Verification</h3>
            <p>This profile is cryptographically linked to the DID above.</p>
            <p>Anyone can verify messages signed by this identity.</p>
        </div>
    </div>
</body>
</html>"#,
                username,
                username,
                identity.did,
                identity.created_at.format("%B %d, %Y"),
                hex::encode(&identity.keys.master.public.as_bytes()[..8]),
                hex::encode(&identity.keys.signing.public.as_bytes()[..8]),
                hex::encode(&identity.keys.authentication.public.as_bytes()[..8])
            );
            
            (StatusCode::OK, Html(html))
        }
        Err(e) => {
            error!("Failed to get identity: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Error loading profile</h1>".to_string()))
        }
    }
}

// API handlers

async fn get_identity(State(state): State<AppState>) -> impl IntoResponse {
    let service = state.identity_service.read().await;
    match service.current().await {
        Ok(identity) => {
            let response = IdentityResponse {
                did: identity.did.to_string(),
                username: generate_username(&identity.did),
                public_keys: PublicKeysInfo {
                    master: hex::encode(identity.keys.master.public.as_bytes()),
                    signing: hex::encode(identity.keys.signing.public.as_bytes()),
                    authentication: hex::encode(identity.keys.authentication.public.as_bytes()),
                },
                created_at: identity.created_at.to_rfc3339(),
            };
            Json(response).into_response()
        }
        Err(e) => {
            error!("Failed to get identity: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to load identity"
            }))).into_response()
        }
    }
}

async fn sign_message(
    State(state): State<AppState>,
    Json(payload): Json<SignRequest>,
) -> impl IntoResponse {
    let service = state.identity_service.read().await;
    
    match service.sign(payload.message.as_bytes()).await {
        Ok(signature) => {
            match service.current().await {
                Ok(identity) => Json(SignResponse {
                    signature: hex::encode(signature),
                    signer_did: identity.did.to_string(),
                }).into_response(),
                Err(e) => {
                    error!("Failed to get identity for signing: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": "Failed to get signer identity"
                    }))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to sign message: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to sign message"
            }))).into_response()
        }
    }
}

async fn create_backup(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let service = state.identity_service.read().await;
    
    let password = payload.get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    
    match service.export_backup(&password).await {
        Ok(backup) => {
            let size = backup.len();
            Json(serde_json::json!({
                "backup": base64::engine::general_purpose::STANDARD.encode(backup),
                "size_bytes": size,
            })).into_response()
        }
        Err(e) => {
            error!("Failed to create backup: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to create backup"
            }))).into_response()
        }
    }
}