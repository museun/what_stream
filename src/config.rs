use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct TagCache {
    pub cache: HashMap<Box<str>, Box<str>>,
}

impl TagCache {
    pub fn get_cache_path() -> Option<PathBuf> {
        dirs::cache_dir().map(|f| f.join("museun").join("what_stream").join("tags_cache.json"))
    }

    pub fn load_cache() -> Self {
        Self::get_cache_path()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|s| serde_json::from_slice(&*s).ok())
            .unwrap_or_default()
    }

    pub fn sync(&self) -> anyhow::Result<()> {
        let path = Self::get_cache_path().with_context(|| "cannot get the cache path")?;
        let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        serde_json::to_writer(&mut writer, self)?;
        Ok(())
    }
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    pub parameters: Parameters,
    pub appearance: Appearance,
}

impl Config {
    pub fn get_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|f| f.join("museun").join("what_stream").join("config.toml"))
    }

    pub const fn default_formatted_toml() -> &'static str {
        r##"
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
tags            = { fg = "#404040" }
"##
    }
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Parameters {
    pub languages: Vec<String>,
    pub query: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Appearance {
    pub glyphs: super::Style,
    pub colors: super::Theme,
}
