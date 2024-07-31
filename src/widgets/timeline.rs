use egui::{pos2, vec2, Color32, Rect, Widget};

use crate::TemplateApp;

pub struct Timeline {
    // pub rect: egui::Rect,
}

pub trait TimelineApi {
    fn shift(&mut self, ticks: f32) {
        todo!()
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui, api: &mut dyn TimelineApi) -> Rect {
        let desired_size = ui.available_width() * vec2(1.0, 0.35);
        let (_, rect) = ui.allocate_space(desired_size);
        let to_screen =
            egui::emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);
        let header_rect =
            Rect::from_min_max(to_screen * pos2(0.0, 0.0), to_screen * pos2(1.0, 0.1));
        let body_rect = Rect::from_min_max(to_screen * pos2(0.0, 0.1), to_screen * pos2(1.0, 1.0));

        let vis = ui.style().noninteractive();
        let color = ui.style().noninteractive().weak_bg_fill;
        let bg_stroke = egui::Stroke {
            width: 1.0,
            ..vis.bg_stroke
        };
        ui.painter().rect(header_rect, 0.0, color, bg_stroke);
        ui.painter().rect(body_rect, 0.0, color, bg_stroke);
        body_rect
    }
}

impl TimelineApi for TemplateApp {}
