use bits_identity::{IdentityService, generate_username};
use bits_consensus::ConsensusService;
use bits_core::Did;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use maud::html;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info};

mod templates;

/// Shared application state
#[derive(Clone)]
struct AppState {
    identity_service: Arc<RwLock<IdentityService>>,
    consensus_service: Arc<ConsensusService>,
    did: Did,
}

/// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Start services
    let identity_service = IdentityService::new("data/identity.db").await?;
    
    // Create or get identity
    let identity = match identity_service.current().await {
        Ok(id) => id,
        Err(_) => {
            info!("Creating new identity...");
            identity_service.create_from_random().await?
        }
    };
    let username = generate_username(&identity.did);
    let consensus_service = Arc::new(ConsensusService::new(identity.did.clone()));

    // Create shared state
    let state = AppState {
        identity_service: Arc::new(RwLock::new(identity_service)),
        consensus_service,
        did: identity.did.clone(),
    };

    // Build router
    let app = Router::new()
        // Static files
        .nest_service("/static", ServeDir::new("crates/node/static"))

        // Web routes
        .route("/", get(home_handler))
        .route("/marketplace", get(marketplace_handler))

        // API routes
        .route("/api/health", get(health_check))

        // Add state and middleware
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Web server listening on http://{}", addr);
    info!("Visit http://localhost:3000 to see your profile: {}", username);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Web handlers

async fn home_handler(State(state): State<AppState>) -> impl IntoResponse {
    let username = generate_username(&state.did);
    let balance = state.consensus_service.get_balance(&state.did).await;
    
    let content = html! {
        div class="space-y-8" {
            // Hero section
            div class="text-center py-12 px-4 sm:px-6 lg:px-8" {
                h1 class="text-4xl font-bold text-gray-900 sm:text-5xl" {
                    "Welcome to Bits"
                }
                p class="mt-3 max-w-2xl mx-auto text-xl text-gray-500" {
                    "Decentralized Identity & Username Marketplace"
                }
            }

            // Identity card
            div class="bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg shadow-xl p-8" {
                div class="text-3xl font-bold mb-2" { (username) }
                div class="font-mono text-sm opacity-90 break-all mb-4" { (state.did.to_string()) }
                div class="text-2xl font-semibold" {
                    (format!("{:.2} BITS", balance as f64 / 1e18))
                }
            }

            // Feature cards
            div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-2" {
                (templates::feature_card(
                    "👤",
                    "Your Identity",
                    "Your decentralized identity is secured on the blockchain",
                    "#",
                    "Learn More"
                ))

                (templates::feature_card(
                    "🛍️",
                    "Marketplace", 
                    "Buy and sell unique usernames",
                    "/marketplace",
                    "Browse Listings"
                ))
            }
        }
    };

    Html(templates::base("Bits - Decentralized Identity",
        templates::container(content)).into_string())
}

async fn marketplace_handler(State(state): State<AppState>) -> impl IntoResponse {
    let balance = state.consensus_service.get_balance(&state.did).await;
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
    </style>
</head>
<body>
    <div class="container">
        <a href="/" style="color: #007bff; text-decoration: none;">← Back to Home</a>
        
        <h1>Username Marketplace</h1>
        <p style="color: #666; margin-bottom: 30px;">Buy and sell unique usernames on the Bits network</p>

        <div class="wallet-info">
            <div>Your Balance: <span class="balance">{:.2} BITS</span></div>
        </div>

        <h2>Available Usernames</h2>
        <div id="listings">
"#, balance as f64 / 1e18);

    if listings.is_empty() {
        html.push_str(r#"<div style="text-align: center; color: #999; padding: 40px;">No usernames listed for sale yet</div>"#);
    } else {
        for listing in listings {
            let username = &listing.username;
            let price = listing.price as f64 / 1e18;
            let seller = &listing.seller_did;
            
            html.push_str(&format!(r#"
                <div class="listing">
                    <div>
                        <div class="username">{}</div>
                        <div class="seller">Seller: {}</div>
                    </div>
                    <div>
                        <div class="price">{:.4} BITS</div>
                    </div>
                </div>
            "#, username, &seller.to_string()[..16], price));
        }
    }

    html.push_str(r#"
        </div>
    </div>
</body>
</html>
    "#);

    Html(html).into_response()
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok"
    }))
}