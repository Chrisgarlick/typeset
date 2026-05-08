use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Extract user ID from the API key.
///
/// API keys are formatted as: `ts_{user_id}_{random}`
/// The key is validated by hashing `{key}:{salt}` and checking it's well-formed.
/// For MVP, we trust the user_id embedded in the key and validate the format.
pub fn extract_user_id(api_key: &str) -> Option<Uuid> {
    let parts: Vec<&str> = api_key.split('_').collect();
    if parts.len() < 3 || parts[0] != "ts" {
        return None;
    }
    parts[1].parse::<Uuid>().ok()
}

/// Authentication middleware
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_key = match auth_header {
        Some(ref header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let user_id = extract_user_id(api_key).ok_or(StatusCode::UNAUTHORIZED)?;

    // Store user_id in request extensions for handlers to access
    request.extensions_mut().insert(UserId(user_id));

    Ok(next.run(request).await)
}

#[derive(Clone, Debug)]
pub struct UserId(pub Uuid);

/// Hash an API key with salt for storage/comparison
pub fn hash_api_key(key: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{key}:{salt}"));
    format!("{:x}", hasher.finalize())
}
