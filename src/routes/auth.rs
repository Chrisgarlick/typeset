use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

/// Authentication middleware — compares the Bearer token to the configured
/// shared secret. Single-tenant model: any caller with the secret has full
/// access; no users, no per-key scoping.
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let presented = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    let expected = state.config.api_token.as_str();

    match presented {
        Some(token) if constant_time_eq(token.as_bytes(), expected.as_bytes()) => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Constant-time byte comparison to avoid timing leaks on the secret.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
