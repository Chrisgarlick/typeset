use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub spaces_key: String,
    pub spaces_secret: String,
    pub spaces_bucket: String,
    pub spaces_region: String,
    pub spaces_endpoint: String,
    pub api_secret_salt: String,
    pub max_render_concurrency: usize,
    pub render_timeout_secs: u64,
    pub output_expiry_days: i64,
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
            spaces_key: env::var("SPACES_KEY").unwrap_or_default(),
            spaces_secret: env::var("SPACES_SECRET").unwrap_or_default(),
            spaces_bucket: env::var("SPACES_BUCKET")
                .unwrap_or_else(|_| "typeset-renders".to_string()),
            spaces_region: env::var("SPACES_REGION")
                .unwrap_or_else(|_| "lon1".to_string()),
            spaces_endpoint: env::var("SPACES_ENDPOINT")
                .unwrap_or_else(|_| "https://lon1.digitaloceanspaces.com".to_string()),
            api_secret_salt: env::var("API_SECRET_SALT")
                .unwrap_or_else(|_| "default-dev-salt".to_string()),
            max_render_concurrency: env::var("MAX_RENDER_CONCURRENCY")
                .unwrap_or_else(|_| "4".to_string())
                .parse()?,
            render_timeout_secs: env::var("RENDER_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            output_expiry_days: env::var("OUTPUT_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            fonts_dir: env::var("FONTS_DIR")
                .unwrap_or_else(|_| "/opt/typeset/fonts".to_string()),
        })
    }
}
