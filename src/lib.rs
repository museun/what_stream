// Note: You must register an application at https://dev.twitch.tv/console

// The client id for your application
pub const WHAT_STREAM_CLIENT_ID: &str = env!("WHAT_STREAM_CLIENT_ID");

// The client secret for your application
pub const WHAT_STREAM_CLIENT_SECRET: &str = env!("WHAT_STREAM_CLIENT_SECRET");

// This is hardcoded to look at specific category, namely 'Science and Tech'
pub const SCIENCE_AND_TECH_CATEGORY: &str = "509670";

mod args;
pub use args::{AppAccess, Args};

mod render;
pub use render::{Entries, Render, Style, Theme};

mod stream;
pub use stream::{fetch_streams, sort_streams, Stream};

mod string;
