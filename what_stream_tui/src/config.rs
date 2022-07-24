use std::path::PathBuf;

use anyhow::Context as _;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Parameters {
    pub languages: Vec<String>,
    pub query: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Appearance {
    pub glyphs: super::Style,
    pub colors: super::Theme,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    pub auth: Auth,
    pub parameters: Parameters,
    pub appearance: Appearance,
}

impl Config {
    // TODO: should these be hardcoded?
    const NAMESPACE: &'static str = "museun";
    const APPLICATION: &'static str = "what_stream";

    const DEFAULT_CONFIG: &'static str = r##"
[auth]
# this should be your `client id`
# you can create one by registering at: https://dev.twitch.tv/console
client_id = ""

# this should be your `client secret`
client_secret = ""

[parameters]
languages    = ["en"]
query        = ["rust"]

[appearance.glyphs]
# TODO explain these
top          = "┌── "
entry_sep    = "│"
end          = "└ "
link         = "├ "
title        = "├ "
continuation = "│ "
stats        = "├ "

[appearance.colors]
# syntax:  { fg: hex-color, bg: hex-color, bold: bool }
# default: { fg: "#C0C0C0", bg: <unset>, bold: false }

# TODO explain these
fringe          = { fg = "#808080", bold = true }
entry           = { fg = "#606060", bold = true }
category        = { fg = "#881798" }
spoken_language = { fg = "#FFFFFF" }
link            = { fg = "#3B78FF", bold = true }
title           = { fg = "#C19C00", bold = true }
uptime          = { fg = "#13A10E" }
viewers         = { fg = "#3A96DD" }
tag             = { fg = "#404040" }
    "##;

    pub fn make_default_config() -> anyhow::Result<()> {
        let path = Self::get_config_path().with_context(|| "cannot find configurationf file")?;
        std::fs::write(&path, Self::default_formatted_toml())?;
        Ok(())
    }

    pub fn make_required_dirs() -> anyhow::Result<()> {
        Self::create_dir(Self::get_cache_dir, || "cannot find cache directory")?;
        Self::create_dir(Self::get_config_dir, || "cannot find config directory")
    }

    fn create_dir(path: fn() -> Option<PathBuf>, err: fn() -> &'static str) -> anyhow::Result<()> {
        let path = path().with_context(err)?;
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn get_config_path() -> Option<PathBuf> {
        Self::get_config_dir().map(|f| f.join("config.toml"))
    }

    pub fn get_config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|f| f.join(Self::NAMESPACE).join(Self::APPLICATION))
    }

    pub fn get_cache_dir() -> Option<PathBuf> {
        dirs::cache_dir().map(|f| f.join(Self::NAMESPACE).join(Self::APPLICATION))
    }

    pub const fn default_formatted_toml() -> &'static str {
        Self::DEFAULT_CONFIG
    }
}
