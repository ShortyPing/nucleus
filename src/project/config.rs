// src/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone)]

pub struct NucleusConfig {
    pub site: SiteConfig,
    pub admin: AdminConfig,
    pub build: BuildConfig,
    pub serve: ServeConfig,
    pub tls: Option<TlsConfig>,
}

#[derive(Deserialize, Clone)]
pub struct SiteConfig {
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct AdminConfig {
    #[serde(default = "default_admin_path")]
    pub path: String,
}

#[derive(Deserialize, Clone)]
pub struct BuildConfig {
    pub command: String,
    pub out_dir: String,
}

#[derive(Deserialize, Clone)]
pub struct ServeConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct TlsConfig {
    pub enabled: bool,
    pub strategy: TlsStrategy,
    pub domain: Option<String>,
    pub certificate_file: Option<String>,
    pub private_key_file: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TlsStrategy {
    Letsencrypt,
    File,
}

fn default_admin_path() -> String {
    "/nucleus".to_string()
}
fn default_port() -> u16 {
    3000
}

impl NucleusConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        toml::from_str(&text).with_context(|| format!("Failed to parse {:?}", path))
    }
}
