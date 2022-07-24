use std::collections::{BTreeSet, HashMap};

use anyhow::Context;
use eframe::{
    egui::{
        style::Margin, Area, CentralPanel, CollapsingHeader, InnerResponse, Layout, Order,
        RichText, ScrollArea, TextEdit, TextFormat, TopBottomPanel, Ui,
    },
    emath::Align,
    epaint::{text::LayoutJob, Color32, Stroke},
};
use indexmap::IndexMap;
use what_stream_core::{AppAccess, Config as Configured, Stream, WhatStream};

#[derive(serde::Serialize, serde::Deserialize)]
struct Auth {
    client_id: String,
    client_secret: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    auth: Auth,
}

impl Config {
    fn load() -> anyhow::Result<Self> {
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

fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let app_access = AppAccess::create(&config.auth.client_id, &config.auth.client_secret)?;
    let app = App::new::<Config>(app_access);

    eframe::run_native(
        "what stream?",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(app)),
    );
}

struct App {
    data: String,
    what_stream: WhatStream,
    results: IndexMap<String, BTreeSet<Stream>>,
    query_map: HashMap<String, usize>,
    showing_error: Option<String>,
}

impl App {
    fn new<C: Configured>(app_access: AppAccess) -> Self {
        let what_stream = WhatStream::new::<&str, _>(
            &[], // "en"
            &[
                what_stream_core::SCIENCE_AND_TECH_CATEGORY,
                what_stream_core::SOFTWARE_AND_GAME_DEV_CATEGORY,
            ],
            app_access,
            &C::get_cache_dir()
                .expect("cache dir")
                .join("tags_cache.json"),
        );

        Self {
            data: String::default(),
            what_stream,
            results: IndexMap::new(),
            query_map: HashMap::new(),
            showing_error: None,
        }
    }
}

impl App {
    fn report_error(&mut self, err: impl std::fmt::Display) {
        self.showing_error.replace(err.to_string());
    }

    fn handle_entry(&mut self) {
        let entry = std::mem::take(&mut self.data);
        if let Err(err) = Lookup::New(self).lookup(entry.split_ascii_whitespace()) {
            self.report_error(err)
        }
    }

    fn maybe_display_error(&mut self, ctx: &eframe::egui::Context) {
        let err = match &self.showing_error {
            Some(err) => err,
            None => return,
        };

        let InnerResponse { inner: done, .. } = Area::new(err)
            .movable(false)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                eframe::egui::containers::Frame::canvas(&<_>::default())
                    .fill(Color32::BLACK)
                    .stroke(Stroke::new(1.0, Color32::DARK_GRAY))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.heading(RichText::new("An error occurred").color(Color32::RED));
                        ui.colored_label(Color32::WHITE, err);

                        ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                            ui.small_button("ok").clicked()
                        })
                        .inner
                    })
                    .inner
            });

        if done {
            self.showing_error.take();
        }
    }

    fn display_results(&mut self, ui: &mut Ui) {
        let mut actions = vec![];
        for (query, streams) in &self.results {
            self.display_entry(query, streams.iter(), &mut actions, ui)
        }

        for action in actions {
            match action {
                Action::Remove(query) => {
                    self.results.remove(&query).unwrap();
                    self.query_map.remove(&query);
                }
                Action::Refresh(query) => {
                    if let Err(err) = Lookup::Refresh(self).lookup(std::iter::once(&*query)) {
                        self.report_error(err)
                    }
                }
            }
        }
    }

    fn display_entry<'i>(
        &self,
        query: &str,
        streams: impl Iterator<Item = &'i Stream> + ExactSizeIterator,
        actions: &mut Vec<Action>,
        ui: &mut Ui,
    ) {
        CollapsingHeader::new(
            LayoutJob::default()
                .simple(query, Color32::GOLD)
                .simple(" (", Color32::GRAY)
                .simple(&streams.len().to_string(), Color32::RED)
                .simple(")", Color32::GRAY),
        )
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.small_button("refresh").clicked() {
                    actions.push(Action::Refresh(query.to_string()));
                }
                if ui.small_button("remove").clicked() {
                    actions.push(Action::Remove(query.to_string()));
                }
            });

            ui.separator();
            for stream in streams {
                self.display_stream(stream, ui);
                ui.separator();
            }
        });
    }

    fn display_stream(&self, stream: &Stream, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.display_info(stream, ui);
            self.display_details(stream, ui);
        });
    }

    fn display_info(&self, stream: &Stream, ui: &mut Ui) {
        let link = format!("https://twitch.tv/{}", stream.user_name);
        ui.hyperlink_to(&link, &link);
        ui.label(RichText::new(&*stream.title).color(Color32::LIGHT_GRAY));
    }

    fn display_details(&self, stream: &Stream, ui: &mut Ui) {
        ui.collapsing(format!("{} details", stream.user_name), |ui| {
            let job = LayoutJob::default()
                .simple("started ", Color32::WHITE)
                .simple(&stream.started_at, Color32::LIGHT_GREEN)
                .simple(" ago, ", Color32::WHITE)
                .simple(&stream.viewer_count.to_string(), Color32::LIGHT_BLUE)
                .simple(" watching", Color32::WHITE);
            ui.label(job);
            self.display_tags(stream, ui);
        });
    }

    fn display_tags(&self, stream: &Stream, ui: &mut Ui) {
        if stream.user_tag_map.is_empty() {
            return;
        }
        let mut tags = stream.user_tag_map.values().collect::<Vec<_>>();
        let len = tags.len();
        tags.sort_unstable();

        let job = tags.into_iter().enumerate().fold(
            LayoutJob::default().simple("tags: ", Color32::WHITE),
            |mut job, (i, c)| {
                if i != 0 && i < len {
                    job = job.simple(", ", Color32::WHITE);
                }
                job.simple(c, Color32::GRAY)
            },
        );
        ui.label(job);
    }
}

enum Lookup<'a> {
    Refresh(&'a mut App),
    New(&'a mut App),
}

impl<'a> Lookup<'a> {
    fn lookup<'b>(&mut self, query: impl Iterator<Item = &'b str> + Clone) -> anyhow::Result<()> {
        let is_new = matches!(self, Self::New(..));

        let app = match self {
            Self::Refresh(app) | Self::New(app) => app,
        };

        let id = app.query_map.len();
        app.query_map.extend(
            query
                .clone()
                .enumerate()
                .map(|(k, v)| (v.to_string(), (k + id))),
        );

        let query = query.collect::<Vec<_>>();

        let mut results = match app.what_stream.fetch_streams(&query) {
            Ok(results) if results.is_empty() => {
                anyhow::bail!(
                    "nothing was found for:\n{}",
                    query.iter().fold(String::new(), |mut a, c| {
                        if !a.is_empty() {
                            a.push('\n');
                        }
                        a.push_str(c);
                        a
                    })
                );
            }
            Ok(results) => results,
            err @ Err(..) => return err.map(drop).map_err(Into::into),
        };

        results.sort_unstable_by(|&(l, ..), &(r, ..)| {
            let get = |i| app.query_map.get(i).copied().unwrap_or(usize::MAX);
            get(l).cmp(&get(r))
        });

        use indexmap::map::Entry;
        for (key, value) in results {
            match app.results.entry(key.to_string()) {
                Entry::Occupied(mut v) if !is_new => {
                    v.get_mut().replace(value);
                }
                Entry::Occupied(mut v) => {
                    v.get_mut().insert(value);
                }
                Entry::Vacant(v) => {
                    v.insert([value].into_iter().collect());
                }
            }
        }

        Ok(())
    }
}

enum Action {
    Remove(String),
    Refresh(String),
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.input_mut().pixels_per_point = 1.5;

        self.maybe_display_error(ctx);

        TopBottomPanel::top("search").show(ctx, |ui| {
            ui.add_sized(ui.available_size(), TextEdit::singleline(&mut self.data));
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    self.display_results(ui);
                })
        });

        if ctx.input().key_pressed(eframe::egui::Key::Enter) {
            self.handle_entry()
        }
    }

    fn warm_up_enabled(&self) -> bool {
        true
    }
}

trait JobExt: Sized {
    fn simple(self, text: &str, color: Color32) -> Self;
}

impl JobExt for LayoutJob {
    fn simple(mut self, text: &str, color: Color32) -> Self {
        use eframe::epaint::{FontFamily, FontId};
        let fmt = TextFormat {
            font_id: FontId::new(14.0, FontFamily::Proportional),
            color,
            ..Default::default()
        };
        self.append(text, 0.0, fmt);
        self
    }
}
