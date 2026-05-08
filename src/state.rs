use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::db::queries::Database;
use crate::storage::spaces::SpacesStorage;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub storage: SpacesStorage,
    pub config: Arc<Config>,
    pub render_semaphore: Arc<Semaphore>,
}
