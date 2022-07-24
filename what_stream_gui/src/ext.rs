use eframe::{
    egui::TextFormat,
    epaint::{text::LayoutJob, Color32},
};

pub trait JobExt: Sized {
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
