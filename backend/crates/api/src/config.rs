use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub log_level: String,
    pub allowed_origin: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 3000,
            log_level: String::from("info,tower_http=debug"),
            allowed_origin: String::from("http://localhost:5173"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config, Box<figment::Error>> {
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Env::prefixed("JMETRO_"))
            .extract()
            .map_err(Box::new)
    }
}
