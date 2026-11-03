use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    /// The old shared secret (`TYPESET_TOKEN`). Still accepted, with full
    /// access, while callers move to Clients keys; unset it to turn it off.
    pub api_token: Option<String>,
    /// Clients' API (`CLIENTS_URL`, e.g. https://clients.kritano.com): brands
    /// and the `ck_` caller keys come from there.
    pub clients_url: Option<String>,
    /// A Clients service key (`CLIENTS_SERVICE_KEY`).
    pub clients_key: String,
    /// Where brands and library files are cached (`CLIENTS_CACHE_DIR`).
    pub clients_cache: String,
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
            api_token: env::var("TYPESET_TOKEN").ok().filter(|t| !t.is_empty()),
            clients_url: env::var("CLIENTS_URL").ok().filter(|u| !u.is_empty()),
            clients_key: env::var("CLIENTS_SERVICE_KEY").unwrap_or_default(),
            clients_cache: env::var("CLIENTS_CACHE_DIR").unwrap_or_else(|_| "/tmp/typeset-clients".to_string()),
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
        .and_then(|c: Config| {
            anyhow::ensure!(
                c.api_token.is_some() || c.clients_url.is_some(),
                "set CLIENTS_URL (and CLIENTS_SERVICE_KEY) or TYPESET_TOKEN: otherwise nobody can call Typeset"
            );
            anyhow::ensure!(c.clients_url.is_none() || !c.clients_key.is_empty(), "CLIENTS_URL is set but CLIENTS_SERVICE_KEY isn't");
            Ok(c)
        })
    }
}
