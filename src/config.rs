use confik::Configuration;
use serde::Deserialize;

#[derive(Debug, Default, Configuration, Clone)]
pub struct AppConfig {
    #[confik(default = "0.0.0.0:8080".to_string())]
    pub server_addr: String,

    #[confik(default = "guest".to_string())]
    pub admin_user: String,

    #[confik(default = "secret".to_string())]
    pub admin_pass: String,

    #[confik(default = "change-me-in-production".to_string())]
    pub jwt_secret: String,

    #[confik(default = false)]
    pub cookie_secure: bool,

    #[confik(from = DbConfig)]
    pub pg: deadpool_postgres::Config,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct DbConfig(deadpool_postgres::Config);

impl From<DbConfig> for deadpool_postgres::Config {
    fn from(value: DbConfig) -> Self {
        value.0
    }
}

impl confik::Configuration for DbConfig {
    type Builder = Option<Self>;
}
