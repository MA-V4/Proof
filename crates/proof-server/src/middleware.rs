use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

pub async fn require_api_key(req: Request, next: Next) -> Response {
    let expected = std::env::var("PROOF_API_KEY").unwrap_or_default();

    // Dev mode: no key set, allow all
    if expected.is_empty() {
        return next.run(req).await;
    }

    // Health check is always public
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }

    let provided = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided == expected {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error":   "Unauthorised",
                "message": "A valid API key is required. Pass it as the X-API-Key header."
            })),
        )
            .into_response()
    }
}