use bits_identity::{IdentityService, generate_username};
use bits_consensus::ConsensusService;
use bits_core::{Component, Did};
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
    consensus_service: Arc<ConsensusService>,
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
    
    // Create consensus service with identity as genesis validator
    let consensus_service = Arc::new(ConsensusService::new(identity.did.clone()));
    info!("Consensus service initialized");
    
    // Create shared state
    let state = AppState {
        identity_service: Arc::new(RwLock::new(identity_service)),
        consensus_service,
    };
    
    // Build router
    let app = Router::new()
        // Web routes
        .route("/", get(home_handler))
        .route("/profile/:username", get(profile_handler))
        .route("/marketplace", get(marketplace_handler))
        
        // API routes
        .route("/api/identity", get(get_identity))
        .route("/api/identity/sign", post(sign_message))
        .route("/api/identity/backup", post(create_backup))
        
        // Blockchain/marketplace API routes
        .route("/api/blockchain/info", get(get_blockchain_info))
        .route("/api/wallet/balance/:did", get(get_balance))
        .route("/api/username/register", post(register_username))
        .route("/api/username/:username", get(get_username_info))
        .route("/api/username/list", post(list_username))
        .route("/api/username/transfer", post(transfer_username))
        .route("/api/marketplace/listings", get(get_marketplace_listings))
        .route("/api/marketplace/search", get(search_usernames))
        
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

// ===== Blockchain/Marketplace API Handlers =====

/// Request to register a username
#[derive(Deserialize)]
struct RegisterUsernameRequest {
    username: String,
}

/// Response for username operations
#[derive(Serialize)]
struct UsernameResponse {
    username: String,
    owner_did: String,
    price: Option<u64>,
    registered_at: u64,
}

/// Request to list a username for sale
#[derive(Deserialize)]
struct ListUsernameRequest {
    username: String,
    price: u64,
}

/// Request to transfer a username
#[derive(Deserialize)]
struct TransferUsernameRequest {
    username: String,
    to_did: String,
    price: Option<u64>,
}

/// Search query parameters
#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn get_blockchain_info(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let info = state.consensus_service.get_blockchain_info().await;
    Json(info)
}

async fn get_balance(
    State(state): State<AppState>,
    Path(did): Path<String>,
) -> impl IntoResponse {
    let did = Did(did);
    let balance = state.consensus_service.get_balance(&did).await;
    
    Json(serde_json::json!({
        "did": did.0,
        "balance": balance,
        "formatted": format!("{} BITS", balance as f64 / 1e18)
    }))
}

async fn register_username(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUsernameRequest>,
) -> impl IntoResponse {
    // Get current identity as the owner
    let identity_service = state.identity_service.read().await;
    let identity = match identity_service.current().await {
        Ok(id) => id,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to get identity: {}", e)
            }))).into_response()
        }
    };
    
    let owner_did = identity.did.clone();
    
    // Register username
    match state.consensus_service.register_username(
        payload.username.clone(),
        owner_did.clone(),
    ).await {
        Ok(tx_id) => {
            Json(serde_json::json!({
                "success": true,
                "username": payload.username,
                "owner_did": owner_did.0,
                "transaction_id": tx_id,
            })).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

async fn get_username_info(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    match state.consensus_service.get_username(&username).await {
        Some(reg) => {
            Json(UsernameResponse {
                username: reg.username,
                owner_did: reg.owner_did.0,
                price: reg.price,
                registered_at: reg.registered_at,
            }).into_response()
        }
        None => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Username not found"
            }))).into_response()
        }
    }
}

async fn list_username(
    State(state): State<AppState>,
    Json(payload): Json<ListUsernameRequest>,
) -> impl IntoResponse {
    // Get current identity as the seller
    let identity_service = state.identity_service.read().await;
    let identity = match identity_service.current().await {
        Ok(id) => id,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to get identity: {}", e)
            }))).into_response()
        }
    };
    
    let seller_did = identity.did.clone();
    
    // List username for sale
    match state.consensus_service.list_username(
        &payload.username,
        seller_did,
        payload.price,
    ).await {
        Ok(tx_id) => {
            Json(serde_json::json!({
                "success": true,
                "username": payload.username,
                "price": payload.price,
                "transaction_id": tx_id,
            })).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

async fn transfer_username(
    State(state): State<AppState>,
    Json(payload): Json<TransferUsernameRequest>,
) -> impl IntoResponse {
    // Get current identity as the sender
    let identity_service = state.identity_service.read().await;
    let identity = match identity_service.current().await {
        Ok(id) => id,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Failed to get identity: {}", e)
            }))).into_response()
        }
    };
    
    let from_did = identity.did.clone();
    let to_did = Did(payload.to_did.clone());
    
    // Transfer username
    match state.consensus_service.transfer_username(
        &payload.username,
        from_did,
        to_did,
        payload.price,
    ).await {
        Ok(tx_id) => {
            Json(serde_json::json!({
                "success": true,
                "username": payload.username,
                "to_did": payload.to_did,
                "transaction_id": tx_id,
            })).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

async fn get_marketplace_listings(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let listings = state.consensus_service.get_marketplace_listings().await;
    Json(listings)
}

async fn search_usernames(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let suggestions = state.consensus_service.search_usernames(&query.q).await;
    Json(serde_json::json!({
        "query": query.q,
        "suggestions": suggestions,
    }))
}

async fn marketplace_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Get current identity for wallet info
    let identity_service = state.identity_service.read().await;
    let identity = match identity_service.current().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get identity: {}", e);
            return Html("<h1>Error loading marketplace</h1>".to_string()).into_response()
        }
    };
    
    let did = identity.did.clone();
    let balance = state.consensus_service.get_balance(&did).await;
    let listings = state.consensus_service.get_marketplace_listings().await;
    
    let mut html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Bits Marketplace</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 40px; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; margin-bottom: 10px; }}
        .wallet-info {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px; }}
        .balance {{ font-size: 24px; font-weight: bold; color: #007bff; }}
        .listing {{ border: 1px solid #e0e0e0; padding: 20px; margin-bottom: 15px; border-radius: 8px; display: flex; justify-content: space-between; align-items: center; }}
        .listing:hover {{ background: #f8f9fa; }}
        .username {{ font-size: 20px; font-weight: 600; color: #333; }}
        .price {{ font-size: 18px; color: #28a745; font-weight: 500; }}
        .seller {{ color: #666; font-size: 14px; margin-top: 5px; }}
        .buy-button {{ background: #007bff; color: white; border: none; padding: 10px 20px; border-radius: 6px; cursor: pointer; font-size: 16px; }}
        .buy-button:hover {{ background: #0056b3; }}
        .search-box {{ width: 100%; padding: 12px; font-size: 16px; border: 1px solid #ddd; border-radius: 6px; margin-bottom: 20px; }}
        .register-form {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px; }}
        .form-group {{ margin-bottom: 15px; }}
        input[type="text"], input[type="number"] {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; }}
        .submit-button {{ background: #28a745; color: white; border: none; padding: 10px 20px; border-radius: 6px; cursor: pointer; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🛍️ Bits Username Marketplace</h1>
        
        <div class="wallet-info">
            <div>Wallet: {}</div>
            <div class="balance">{} BITS</div>
        </div>
        
        <div class="register-form">
            <h3>Register New Username</h3>
            <div class="form-group">
                <input type="text" id="new-username" placeholder="Enter desired username (3-30 chars, lowercase, hyphens allowed)">
            </div>
            <button class="submit-button" onclick="registerUsername()">Check Availability & Register</button>
            <div id="register-result"></div>
        </div>
        
        <h2>Available Usernames</h2>
        <input type="text" class="search-box" placeholder="Search usernames..." onkeyup="searchUsernames(this.value)">
        
        <div id="listings">
"#, did.0, balance as f64 / 1e18);
    
    if listings.is_empty() {
        html.push_str("<p>No usernames listed for sale yet.</p>");
    } else {
        for listing in listings {
            html.push_str(&format!(r#"
            <div class="listing">
                <div>
                    <div class="username">{}</div>
                    <div class="seller">Seller: {}</div>
                </div>
                <div>
                    <div class="price">{} BITS</div>
                    <button class="buy-button" onclick="buyUsername('{}', {})">Buy Now</button>
                </div>
            </div>
            "#, 
            listing.username, 
            &listing.seller_did.0[..16], 
            listing.price as f64 / 1e18,
            listing.username,
            listing.price
            ));
        }
    }
    
    html.push_str(r#"
        </div>
    </div>
    
    <script>
        async function registerUsername() {
            const username = document.getElementById('new-username').value;
            const resultDiv = document.getElementById('register-result');
            
            if (!username) {
                resultDiv.innerHTML = '<p style="color: red;">Please enter a username</p>';
                return;
            }
            
            try {
                const response = await fetch('/api/username/register', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({username})
                });
                
                const data = await response.json();
                
                if (response.ok) {
                    resultDiv.innerHTML = `<p style="color: green;">✅ Successfully registered ${username}!</p>`;
                    document.getElementById('new-username').value = '';
                    setTimeout(() => location.reload(), 2000);
                } else {
                    resultDiv.innerHTML = `<p style="color: red;">❌ ${data.error}</p>`;
                }
            } catch (error) {
                resultDiv.innerHTML = `<p style="color: red;">❌ Error: ${error.message}</p>`;
            }
        }
        
        async function buyUsername(username, price) {
            if (!confirm(`Buy ${username} for ${price / 1e18} BITS?`)) return;
            
            alert('Purchase functionality coming soon!');
        }
        
        async function searchUsernames(query) {
            if (!query) return;
            
            try {
                const response = await fetch(`/api/marketplace/search?q=${encodeURIComponent(query)}`);
                const data = await response.json();
                
                console.log('Search suggestions:', data.suggestions);
            } catch (error) {
                console.error('Search error:', error);
            }
        }
    </script>
</body>
</html>
    "#);
    
    Html(html).into_response()
}