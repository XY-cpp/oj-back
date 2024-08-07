use lazy_static::lazy_static;
use serde::Deserialize;
use toml;

lazy_static! {
  pub static ref config: Settings = Settings::default();
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
  pub server: Server,
  pub mysql: Database,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
  pub host: String,
  pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub database: String,
}

impl Default for Settings {
  fn default() -> Self {
    let toml_data = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str::<Settings>(&toml_data).unwrap()
  }
}
