use eyre::WrapErr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_bind")]
    pub bind_address: String,
    #[serde(default = "default_anki_url")]
    pub anki_connect_url: String,
    #[serde(default = "default_retention")]
    pub desired_retention: f32,
}

fn default_bind() -> String {
    "127.0.0.1:3000".into()
}

fn default_anki_url() -> String {
    "http://127.0.0.1:8765".into()
}

fn default_retention() -> f32 {
    0.9
}

#[derive(Debug, Deserialize)]
pub struct DeckConfigs {
    pub deck: Vec<DeckConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeckConfig {
    pub name: String,
    pub query: String,
    pub fields: FieldMapping,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldMapping {
    pub word: String,
    pub reading: String,
    pub meaning: String,
}

impl DeckConfigs {
    pub fn load(path: &str) -> eyre::Result<Self> {
        let content = std::fs::read_to_string(path).wrap_err_with(|| format!("reading {path}"))?;
        toml::from_str(&content).wrap_err("parsing deck config toml")
    }
}
