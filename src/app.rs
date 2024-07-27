use eframe::App;
use egui::{epaint::PathStroke, mutex::Mutex, pos2, vec2, Color32, Frame, Pos2, Rect, Ui};

use crate::{
    buffer::Buffer,
    data::{Chunk, Data},
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

    paused: bool,

    #[serde(skip)] // This how you opt-out of serialization of a field
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
            value: 2.7,
            paused: false,
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

            self.draw_line(ui);

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
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn clear(&mut self) {
        self.max_id = 1;
        self.buf.clear();
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn draw(&mut self, data: &[u8]) {
        self.buf.push(Chunk::new(self.max_id, Vec::from(data)));
        self.buf.set_max_id(self.max_id);
        self.max_id += 1;
        self.cursor_pos = self.max_id;
    }

    fn draw_line(&mut self, ui: &mut Ui) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.paused = !self.paused;
            }
            if self.paused {
                let scroll_value = i.smooth_scroll_delta.y;
                let raw_scroll_value = i.raw_scroll_delta.y;
                if raw_scroll_value < 0.0 {
                    self.cursor_pos = self.max_id.min(self.cursor_pos + 1);
                } else if raw_scroll_value > 0.0 {
                    self.cursor_pos = self.cursor_pos.checked_sub(1).unwrap_or(0);
                }
                Self::log(&format!(
                    "scroll {}, raw {}",
                    scroll_value, raw_scroll_value
                ));
            }
        });
        let view = View {
            start: self.cursor_pos as isize - 10,
            end: self.cursor_pos as isize,
        };
        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let desired_size = ui.available_width() * vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);
            let to_screen = egui::emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                rect,
            );
            let data = self.buf.get_data(&view);
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
            ui.painter().extend(shapes);
        });
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
