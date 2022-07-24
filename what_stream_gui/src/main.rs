use what_stream_core::AppAccess;

mod action;
mod app;
mod config;
mod ext;

fn main() -> anyhow::Result<()> {
    let config = config::Config::load()?;
    let app_access = AppAccess::create(&config.auth.client_id, &config.auth.client_secret)?;
    let app = app::App::new::<config::Config>(app_access);

    eframe::run_native(
        "what stream?",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(app)),
    );
}
