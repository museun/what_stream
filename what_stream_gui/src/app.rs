use std::collections::{BTreeSet, HashMap};

use eframe::{
    self,
    egui::*,
    epaint::{text::LayoutJob, Color32, Stroke},
};
use indexmap::IndexMap;
use what_stream_core::{AppAccess, Config as Configured, Stream, WhatStream};

use crate::{
    action::{Action, Lookup},
    ext::JobExt,
};

pub struct App {
    pub data: String,
    pub what_stream: WhatStream,
    pub results: IndexMap<String, BTreeSet<Stream>>,
    pub query_map: HashMap<String, usize>,
    pub showing_error: Option<String>,
}

impl App {
    pub fn new<C: Configured>(app_access: AppAccess) -> Self {
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
                    .inner_margin(style::Margin::same(10.0))
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
