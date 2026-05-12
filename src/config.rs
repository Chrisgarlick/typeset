use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub api_token: String,
    pub rate_limit_per_min: usize,
    pub max_render_concurrency: usize,
    pub render_timeout_secs: u64,
    pub fonts_dir: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3200".to_string())
                .parse()?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            database_url: env::var("DATABASE_URL")?,
            api_token: env::var("TYPESET_TOKEN")
                .map_err(|_| anyhow::anyhow!("TYPESET_TOKEN env var must be set"))?,
            rate_limit_per_min: env::var("TYPESET_RATE_LIMIT_PER_MIN")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            max_render_concurrency: env::var("MAX_RENDER_CONCURRENCY")
                .unwrap_or_else(|_| "4".to_string())
                .parse()?,
            render_timeout_secs: env::var("RENDER_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            fonts_dir: env::var("FONTS_DIR")
                .unwrap_or_else(|_| "/opt/typeset/fonts".to_string()),
        })
    }
}
