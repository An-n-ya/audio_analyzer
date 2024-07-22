#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{cell::RefCell, rc::Rc};

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

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Ok(Box::new(eframe_template::TemplateApp::new(cc)))),
            )
            .await;

        wasm_bindgen_futures::spawn_local(async {
            setup_audio_device().await;
        });

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

async fn setup_audio_device() {
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

        animate_limited(
            move || {
                analyzer.get_byte_time_domain_data(&mut buffer);

                web_sys::console::debug_1(&JsValue::from_f64(buffer[0] as f64));
                web_sys::console::debug_1(&JsValue::from_f64(buffer[1] as f64));
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
