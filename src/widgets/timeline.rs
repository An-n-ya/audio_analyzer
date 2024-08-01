use std::ops::Sub;

use egui::{epaint::PathStroke, pos2, vec2, Color32, Rect, Widget};

use crate::TemplateApp;

pub struct Timeline {
    // pub rect: egui::Rect,
}

pub trait TimelineApi {
    fn shift(&mut self, ticks: f32) {
        todo!()
    }
    fn zoom(&mut self, factor: f32) {
        todo!()
    }
    fn get_calibration(&self) -> f32 {
        30.0
    }
    fn get_time_range(&self) -> (f32, f32) {
        (0.0, 1.0)
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
        let header_rect = Rect::from_min_max(
            to_screen * pos2(0.0, 0.0),
            pos2(rect.max.x, 20.0 + rect.min.y),
        );
        let body_rect = Rect::from_min_max(
            pos2(rect.min.x, 20.0 + rect.min.y),
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
            let mut start = pos2(item.pos, 0.7);
            let end = pos2(item.pos, 1.0);
            if cnt % 5 == 0 {
                start.y = 0.3;
                // TODO: show time
            }
            ui.painter().line_segment(
                [to_screen * end, to_screen * start],
                PathStroke::new(0.5, color),
            );
            cnt += 1;
        }
    }
}

impl TimelineApi for TemplateApp {}

struct Step {
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
impl<'a> Iterator for StepIter<'a> {
    type Item = StepIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= 1.0 - f32::EPSILON {
            None
        } else {
            let pos = self.cur;
            self.cur = self.cur + 1.0 / self.step.calibration;
            Some(StepIterItem {
                pos,
                time: self.step.time_range.0
                    + pos * (self.step.time_range.1 - self.step.time_range.0),
            })
        }
    }
}
