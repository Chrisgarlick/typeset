use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

const WINDOW: Duration = Duration::from_secs(60);

/// Per-IP sliding-window rate limiter. Memory-resident, no external store.
#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    max_per_window: usize,
}

impl RateLimiter {
    pub fn new(max_per_window: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            max_per_window,
        }
    }

    /// Returns Ok(()) if the request is allowed, or Err(seconds_until_retry)
    /// if the caller has exceeded the window.
    fn check(&self, ip: IpAddr) -> Result<(), u64> {
        let now = Instant::now();
        let mut map = self.state.lock().unwrap();
        let entries = map.entry(ip).or_default();
        entries.retain(|&t| now.duration_since(t) < WINDOW);

        if entries.len() >= self.max_per_window {
            let oldest = entries.first().copied().unwrap_or(now);
            let retry_after = WINDOW
                .saturating_sub(now.duration_since(oldest))
                .as_secs()
                .max(1);
            return Err(retry_after);
        }

        entries.push(now);
        Ok(())
    }
}

/// Axum middleware. Pulls the limiter from AppState and the caller's IP from
/// X-Forwarded-For (when behind nginx) or the direct connection.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let ip = extract_client_ip(request.headers(), addr);
    match state.rate_limiter.check(ip) {
        Ok(()) => Ok(next.run(request).await),
        Err(retry_after) => {
            let body = serde_json::json!({
                "error": format!("Rate limit exceeded — try again in {retry_after}s"),
                "status": 429,
            });
            let mut resp =
                (StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
            resp.headers_mut().insert(
                "retry-after",
                retry_after.to_string().parse().unwrap(),
            );
            Err(resp)
        }
    }
}

fn extract_client_ip(headers: &HeaderMap, addr: SocketAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or_else(|| addr.ip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn allows_under_limit() {
        let rl = RateLimiter::new(3);
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        assert!(rl.check(ip).is_ok());
        assert!(rl.check(ip).is_ok());
        assert!(rl.check(ip).is_ok());
    }

    #[test]
    fn blocks_over_limit() {
        let rl = RateLimiter::new(2);
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        assert!(rl.check(ip).is_ok());
        assert!(rl.check(ip).is_ok());
        assert!(rl.check(ip).is_err());
    }

    #[test]
    fn limits_are_per_ip() {
        let rl = RateLimiter::new(1);
        let a = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        let b = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
        assert!(rl.check(a).is_ok());
        assert!(rl.check(b).is_ok());
        assert!(rl.check(a).is_err());
    }
}
