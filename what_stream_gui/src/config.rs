use anyhow::Context as _;
use what_stream_core::Config as Configured;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub auth: Auth,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let file = Self::get_config_dir()
            .with_context(|| "cannot get config dir")?
            .join(Self::config_file_name());

        let data = std::fs::read(file)?;
        Ok(toml::from_slice(&data)?)
    }
}

impl Configured for Config {
    const NAMESPACE: &'static str = "museun";
    const APPLICATION: &'static str = "what_stream"; // TODO different name

    fn config_file_name() -> &'static str {
        "config.toml"
    }
}
