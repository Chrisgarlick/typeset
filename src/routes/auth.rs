use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::error::AppError;
use crate::state::AppState;

/// Who is calling, put in the request's extensions by `auth_middleware`.
#[derive(Clone, Debug)]
pub enum Caller {
    /// The old shared secret (`TYPESET_TOKEN`): every client.
    Legacy,
    /// A `ck_` caller key from Clients, possibly limited to some clients.
    Key(crate::clients::Caller),
}

impl Caller {
    /// Whether this caller may use a client's brand.
    pub fn may(&self, slug: &str) -> bool {
        match self {
            Caller::Legacy => true,
            Caller::Key(c) => c.may(slug),
        }
    }
}

/// Authentication: `Authorization: Bearer ck_…`, a caller key checked with
/// Clients (cached 60s, so a revoked key stops working within a minute), or,
/// while it's still set, the old shared `TYPESET_TOKEN`.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let presented = header
        .as_deref()
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::trim);

    let caller = match (presented, &state.config.api_token, &state.brands) {
        (Some(token), Some(expected), _)
            if constant_time_eq(token.as_bytes(), expected.as_bytes()) =>
        {
            Caller::Legacy
        }
        (Some(token), _, Some(brands)) if token.starts_with("ck_") => {
            match brands.authenticate(header.as_deref()).await {
                Ok(c) => Caller::Key(c),
                Err(crate::clients::Error::Unauthorized) => {
                    return StatusCode::UNAUTHORIZED.into_response()
                }
                Err(e) => return AppError::from(e).into_response(),
            }
        }
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    request.extensions_mut().insert(caller);
    next.run(request).await
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
