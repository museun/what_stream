// Note: You must register an application at https://dev.twitch.tv/console

// The client id for your application
pub const WHAT_STREAM_CLIENT_ID: &str = env!("WHAT_STREAM_CLIENT_ID");

// The client secret for your application
pub const WHAT_STREAM_CLIENT_SECRET: &str = env!("WHAT_STREAM_CLIENT_SECRET");

// TODO allow for custom categories
// TODO provide a utlity for looking up category ideas for a query

// This is hardcoded to look at specific category, namely 'Science and Tech'
pub const SCIENCE_AND_TECH_CATEGORY: &str = "509670";

// This is hardcoded to look at specific category, namely 'Software and Game development'
pub const SOFTWARE_AND_GAME_DEV_CATEGORY: &str = "1469308723";

mod args;
pub use args::{AppAccess, Args};

mod render;
pub use render::{Appearance, Config, Demo, Entries, Parameters, Render, Style, TagCache, Theme};

mod stream;
pub use stream::{fetch_streams, sort_streams, Stream};

mod string;
