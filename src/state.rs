use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::db::queries::Database;
use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
    pub render_semaphore: Arc<Semaphore>,
    pub rate_limiter: RateLimiter,
}
