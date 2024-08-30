use eframe::App;
use egui::{epaint::PathStroke, pos2, Color32, Frame, Pos2, Rect, Ui};
use js_sys::Date;

use crate::{
    buffer::Buffer,
    data::Chunk,
    widgets::timeline::{Timeline, TimelineApi},
    Log,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    buf: Buffer,

    max_id: usize,

    cursor_pos: usize,
    cursor_time: f64,

    data: Option<Vec<u8>>,
    range: Option<(f32, f32)>,

    recording_start_time: f64,

    chunk_num: usize,

    paused: bool, // This how you opt-out of serialization of a field
    value: f32,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RingBuffer {
    pub head: usize,
    pub size: usize,
    buf: Vec<Vec<u8>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct View {
    pub start: isize,
    pub end: isize,
}

impl Default for RingBuffer {
    fn default() -> Self {
        let mut buf = vec![];
        for _ in 0..Self::BUF_SIZE {
            buf.push(vec![]);
        }
        Self {
            head: 0,
            size: 0,
            buf,
        }
    }
}

impl RingBuffer {
    const BUF_SIZE: usize = 20;
    pub fn push(&mut self, data: Vec<u8>) {
        let ind = if self.head == 0 {
            Self::BUF_SIZE - 1
        } else {
            self.head
        };
        self.buf[ind] = data;
        self.head += 1;
        self.head %= Self::BUF_SIZE;
    }
    pub fn len(&self) -> usize {
        Self::BUF_SIZE * self.size
    }
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
    pub fn get(&mut self, ind: usize) -> u8 {
        assert!(self.size > 0);
        let buf_ind = ind / self.size;
        let sub_ind = ind % self.size;
        let buf_ind = (self.head + buf_ind) % Self::BUF_SIZE;
        while buf_ind >= self.buf.len() {
            self.buf.push(vec![]);
        }
        let selected_buf = &self.buf[buf_ind];
        if selected_buf.len() == 0 {
            return 128;
        }
        self.buf[buf_ind][sub_ind]
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            buf: Buffer::new(),
            max_id: 1,
            cursor_pos: 1,
            recording_start_time: 0.0,
            cursor_time: 0.0,
            chunk_num: 10,
            data: None,
            range: None,
            value: 2.7,
            paused: true,
        }
    }
}

impl App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            self.handle_input(ui);

            let mut timeline = Timeline::new();
            let body_rect = timeline.show(ui, self);
            self.draw_line(ui, body_rect);

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

impl Log for TemplateApp {
    fn name() -> &'static str {
        "TemplateAPP"
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    pub fn clear(&mut self) {
        self.max_id = 1;
        self.buf.clear();
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn update(&mut self, data: &[u8]) {
        // don't use Date::now(), use calculated time instead
        let current_time = Date::now();
        let time = current_time - self.recording_start_time + self.cursor_time;
        // Self::log(&format!(
        //     "current_time {}, update time {}",
        //     current_time, time
        // ));
        self.buf
            .push(Chunk::new(self.max_id, Vec::from(data), time as f32));
        self.buf.set_max_id(self.max_id);
        self.max_id += 1;
        self.cursor_pos = self.max_id;
    }

    fn handle_input(&mut self, ui: &mut Ui) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.paused = !self.paused;
                Self::log("paused changed");
                if !self.paused {
                    self.recording_start_time = Date::now();
                    Self::log(&format!("recording time: {}", self.recording_start_time));
                }
            }
            if self.paused {
                let raw_scroll_value = i.raw_scroll_delta.y;
                if raw_scroll_value < 0.0 {
                    self.cursor_pos = self.max_id.min(self.cursor_pos + 1);
                } else if raw_scroll_value > 0.0 {
                    self.cursor_pos = self.cursor_pos.checked_sub(1).unwrap_or(0);
                }
            }
        });
    }

    fn draw_line(&mut self, ui: &mut Ui, rect: Rect) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };
        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let to_screen = egui::emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                rect,
            );
            if let Some(data) = &self.data {
                let n = data.len();
                let mut shapes = vec![];
                let points: Vec<Pos2> = (0..n)
                    .map(|i| {
                        let t = i as f64 / (n as f64);
                        let y = (data[i] as f64 - 128.0) / 128.0;
                        to_screen * pos2(t as f32, y as f32)
                    })
                    .collect();
                shapes.push(egui::epaint::Shape::line(
                    points,
                    PathStroke::new(2.0, color),
                ));
                let bar = [to_screen * pos2(0.5, -1.0), to_screen * pos2(0.5, 1.0)];
                shapes.push(egui::epaint::Shape::line_segment(
                    bar,
                    PathStroke::new(1.0, color),
                ));
                ui.painter().extend(shapes);
            }
        });
    }
}

impl TimelineApi for TemplateApp {
    fn flush_data(&mut self) {
        let view = View {
            start: self.cursor_pos as isize - self.chunk_num as isize,
            end: self.cursor_pos as isize,
        };
        let data_range = self.buf.get_data(&view);
        self.data = Some(data_range.data);
        self.range = data_range.time_range;
    }

    fn time_range_span(&self) -> f32 {
        self.chunk_num as f32 * 16.67
    }

    fn get_time_range(&self) -> (f32, f32) {
        if let Some((start, end)) = self.range {
            if start == end {
                (0.0, self.time_range_span())
            } else {
                // let start = (start - 16.67).max(0.0);
                (start, start + self.time_range_span())
            }
        } else {
            (0.0, self.time_range_span())
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
