use axum::response::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PongResponse {
    status: String,
}

pub async fn ping() -> Json<PongResponse> {
    Json(PongResponse {
        status: "OK".to_string(),
    })
}
