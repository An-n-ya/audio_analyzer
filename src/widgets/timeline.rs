use std::ops::Sub;

use egui::{epaint::PathStroke, pos2, vec2, Align2, Color32, FontId, Rect, TextStyle, Widget};

use crate::{Log, TemplateApp};

pub struct Timeline {
    // pub rect: egui::Rect,
}

impl Log for Timeline {
    fn name() -> &'static str {
        "Timeline"
    }
}

pub trait TimelineApi {
    fn shift(&mut self, ticks: f32) {
        todo!()
    }
    fn zoom(&mut self, factor: f32) {
        todo!()
    }
    fn get_calibration(&self) -> f32 {
        5.0
    }
    fn get_time_range(&self) -> (f32, f32) {
        (0.0, self.time_range_span())
    }
    fn flush_data(&mut self);
    fn time_range_span(&self) -> f32;
}

impl Timeline {
    const HEADER_HEIGHT: f32 = 30.0;
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui, api: &mut dyn TimelineApi) -> Rect {
        api.flush_data();

        let desired_size = ui.available_width() * vec2(1.0, 0.35);
        let (_, rect) = ui.allocate_space(desired_size);
        let to_screen =
            egui::emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);
        let header_rect = Rect::from_min_max(
            to_screen * pos2(0.0, 0.0),
            pos2(rect.max.x, Self::HEADER_HEIGHT + rect.min.y),
        );
        let body_rect = Rect::from_min_max(
            pos2(rect.min.x, Self::HEADER_HEIGHT + rect.min.y),
            to_screen * pos2(1.0, 1.0),
        );

        let vis = ui.style().noninteractive();
        let color = ui.style().noninteractive().weak_bg_fill;
        let bg_stroke = egui::Stroke {
            width: 1.0,
            ..vis.bg_stroke
        };
        ui.painter().rect(header_rect, 0.0, color, bg_stroke);
        ui.painter().rect(body_rect, 0.0, color, bg_stroke);
        Self::draw_header(ui, header_rect, api);
        body_rect
    }

    pub fn draw_header(ui: &mut egui::Ui, rect: Rect, api: &mut dyn TimelineApi) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(96)
        } else {
            Color32::from_black_alpha(140)
        };
        let step = Step {
            pixel_width: rect.width(),
            calibration: api.get_calibration(),
            time_range: api.get_time_range(),
        };
        let to_screen =
            egui::emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);
        let mut cnt = 0;
        // ui.painter().line_segment(
        //     [to_screen * pos2(0.2, 0.0), to_screen * pos2(0.2, 1.0)],
        //     PathStroke::new(0.5, color),
        // );
        for item in step.iter() {
            let mut start = pos2(item.pos, 0.8);
            let end = pos2(item.pos, 1.0);
            if cnt % 5 != 0 {}
            if cnt % 5 == 0 {
                start.y = 0.5;
                let font_id = FontId::new(10.0, egui::FontFamily::Monospace);
                let pos = to_screen * pos2(item.pos, 0.05);
                let text = format!("{}", item.time.floor());
                ui.painter()
                    .text(pos, Align2::CENTER_TOP, text, font_id, color);
            }
            ui.painter().line_segment(
                [to_screen * end, to_screen * start],
                PathStroke::new(0.5, color),
            );
            cnt += 1;
        }
    }
}

struct Step {
    pixel_width: f32,
    calibration: f32,
    time_range: (f32, f32),
}

struct StepIter<'a> {
    step: &'a Step,
    cur: f32,
}

impl Step {
    pub fn iter<'a>(&'a self) -> StepIter<'a> {
        StepIter {
            step: self,
            cur: 0.0,
        }
    }
}

struct StepIterItem {
    pos: f32,
    time: f32,
}
impl<'a> StepIter<'a> {
    fn time(&self) -> f32 {
        self.step.time_range.0 + self.cur * (self.step.time_range.1 - self.step.time_range.0)
    }
}
impl<'a> Log for StepIter<'a> {
    fn name() -> &'static str {
        "StepIter"
    }
}
impl<'a> Iterator for StepIter<'a> {
    type Item = StepIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        let time_range = self.step.time_range;
        let time_span = time_range.1 - time_range.0;
        // Self::log(&format!("time_range: {}, {}", time_range.0, time_range.1));
        assert!(time_span > 0.0);
        let resolution = 1.0 / self.step.pixel_width;
        let time_resolution = time_span * resolution;
        while self.cur < 1.0 {
            let remain = self.time() % self.step.calibration;
            if remain < time_resolution {
                let pos = self.cur;
                let time = self.time();
                self.cur += resolution;
                return Some(StepIterItem { pos, time });
            }
            self.cur += resolution;
        }
        return None;
    }
}
