use eframe::App;
use egui::{epaint::PathStroke, pos2, vec2, Color32, Frame, Pos2, Rect, Ui};
use wasm_bindgen::JsValue;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    data: Vec<u8>,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            data: vec![],
            value: 2.7,
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

    pub fn draw(&mut self, data: &[u8]) {
        self.data = Vec::from(data);
        // if data.iter().all(|n| *n == 128) {
        //     web_sys::console::debug_1(&JsValue::from_bool(true));
        // } else {
        //     web_sys::console::debug_1(&JsValue::from_bool(false));
        // }
        // web_sys::console::debug_1(&JsValue::from_f64(data[0] as f64));
        // web_sys::console::debug_1(&JsValue::from_f64(data[1] as f64));
    }

    fn draw_line(&self, ui: &mut Ui) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };
        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let desired_size = ui.available_width() * vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);
            let to_screen = egui::emath::RectTransform::from_to(
                Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                rect,
            );
            let n = self.data.len();
            let mut shapes = vec![];
            let points: Vec<Pos2> = (0..n)
                .map(|i| {
                    let t = i as f64 / (n as f64);
                    let y = (self.data[i] as f64 - 128.0) / 128.0;
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
