#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe_template::TemplateApp;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use eframe::wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{AudioContext, MediaStream, MediaStreamConstraints};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(eframe_template::TemplateApp::new(cc)))),
    )
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    /// Installs a panic hook, then returns.
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Redirect [`log`] message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Call this once from JavaScript to start your app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.runner
            .start(
                canvas_id,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
            )
            .await
    }

    // The following are optional:

    /// Shut down eframe and clean up resources.
    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    #[wasm_bindgen]
    pub fn update(&mut self, data: &[u8]) {
        if let Some(ref mut app) = self.runner.app_mut::<TemplateApp>() {
            app.draw(data);
        }
    }
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        if let Some(ref mut app) = self.runner.app_mut::<TemplateApp>() {
            app.clear();
        }
    }
    #[wasm_bindgen]
    pub fn is_paused(&mut self) -> bool {
        if let Some(ref mut app) = self.runner.app_mut::<TemplateApp>() {
            app.is_paused()
        } else {
            false
        }
    }

    /// The JavaScript can check whether or not your app has crashed:
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.message())
    }

    #[wasm_bindgen]
    pub fn panic_callstack(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.callstack())
    }
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let handle = WebHandle::new();

    wasm_bindgen_futures::spawn_local(async move {
        let start_result = handle.start("the_canvas_id").await;

        setup_audio_device(handle).await;

        // Remove the loading text and spinner:
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

async fn setup_audio_device(mut handle: WebHandle) {
    handle.clear();
    let navigator: web_sys::Navigator = web_sys::window()
        .and_then(|w| Some(w.navigator()))
        .expect("cannot find navigator");
    let media = navigator
        .media_devices()
        .and_then(|devices| {
            let res = devices.get_user_media_with_constraints(
                MediaStreamConstraints::new().audio(&JsValue::from_bool(true)),
            );
            res
        })
        .expect("cannot find device on your browser");

    let on_success = Closure::wrap(Box::new(move |value: JsValue| {
        let media_stream = MediaStream::from(value);
        let audio_ctx = AudioContext::new().expect("cannot instantiate AudioContext");
        let source = audio_ctx
            .create_media_stream_source(&media_stream)
            .expect("cannot create media stream source");
        let analyzer = audio_ctx
            .create_analyser()
            .expect("analyzer node creating failed");
        analyzer.set_fft_size(2048);
        let buffer_size = analyzer.frequency_bin_count();
        source
            .connect_with_audio_node(&analyzer)
            .expect("connect to analyzer failed");
        let mut buffer = vec![0; buffer_size as usize];
        let mut handle = handle.clone();

        let mut pause_state = false;

        animate_limited(
            move || {
                if pause_state && !handle.is_paused() {
                    let _ = audio_ctx.resume();
                    pause_state = false;
                } else if !pause_state && handle.is_paused() {
                    let _ = audio_ctx.suspend();
                    pause_state = true;
                    return;
                }
                if pause_state {
                    return;
                }
                analyzer.get_byte_time_domain_data(&mut buffer);
                handle.update(&buffer);
            },
            60,
        );
    }) as Box<dyn FnMut(JsValue)>);

    let _ = media.then(&on_success);
    on_success.forget();
}

fn animate_limited(mut draw_frame: impl FnMut() + 'static, max_fps: i32) {
    // Based on:
    // https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html#srclibrs

    // https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
    let animate_cb = Rc::new(RefCell::new(None));
    let animate_cb2 = animate_cb.clone();

    let timeout_cb = Rc::new(RefCell::new(None));
    let timeout_cb2 = timeout_cb.clone();

    let w = window();
    *timeout_cb2.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        request_animation_frame(&w, animate_cb.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    let w2 = window();
    *animate_cb2.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        draw_frame();

        set_timeout(&w2, timeout_cb.borrow().as_ref().unwrap(), 1000 / max_fps);
    }) as Box<dyn FnMut()>));

    request_animation_frame(&window(), animate_cb2.borrow().as_ref().unwrap());
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(window: &web_sys::Window, f: &Closure<dyn FnMut()>) -> i32 {
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

fn set_timeout(window: &web_sys::Window, f: &Closure<dyn FnMut()>, timeout_ms: i32) -> i32 {
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            timeout_ms,
        )
        .expect("should register `setTimeout` OK")
}
