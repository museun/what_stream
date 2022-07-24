// Note: You must register an application at https://dev.twitch.tv/console

// TODO allow for custom categories
// TODO provide a utlity for looking up category ideas for a query

// This is hardcoded to look at specific category, namely 'Science and Tech'
pub const SCIENCE_AND_TECH_CATEGORY: &str = "509670";

// This is hardcoded to look at specific category, namely 'Software and Game development'
pub const SOFTWARE_AND_GAME_DEV_CATEGORY: &str = "1469308723";

mod what_stream;
use std::path::PathBuf;

pub use what_stream::WhatStream;

mod stream;
pub use stream::Stream;

mod tags_cache;
pub use tags_cache::TagCache;

mod app_access;
pub use app_access::AppAccess;

mod sort;
pub use sort::{Column, Direction, SortAction};

mod data;
pub mod util;

mod http;

pub trait Config {
    const NAMESPACE: &'static str;
    const APPLICATION: &'static str;

    fn config_file_name() -> &'static str;

    fn get_config_path() -> Option<PathBuf> {
        Self::get_config_dir().map(|s| s.join(Self::config_file_name()))
    }

    fn get_config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|f| f.join(Self::NAMESPACE).join(Self::APPLICATION))
    }
    fn get_cache_dir() -> Option<PathBuf> {
        dirs::cache_dir().map(|f| f.join(Self::NAMESPACE).join(Self::APPLICATION))
    }
}
