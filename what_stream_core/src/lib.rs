// Note: You must register an application at https://dev.twitch.tv/console

// TODO allow for custom categories
// TODO provide a utlity for looking up category ideas for a query

// This is hardcoded to look at specific category, namely 'Science and Tech'
pub const SCIENCE_AND_TECH_CATEGORY: &str = "509670";

// This is hardcoded to look at specific category, namely 'Software and Game development'
pub const SOFTWARE_AND_GAME_DEV_CATEGORY: &str = "1469308723";

mod what_stream;
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
mod util;

mod http;
