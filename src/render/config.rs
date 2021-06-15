use std::path::PathBuf;

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
