use js_sys::Math::sin;
use wasm_bindgen::JsValue;
use web_sys::{AudioBuffer, AudioContext};

pub fn sine_buffer(audio_ctx: &AudioContext) -> Result<AudioBuffer, JsValue> {
    let buffer = audio_ctx.create_buffer(
        1,
        (audio_ctx.sample_rate() * 1.0) as u32,
        audio_ctx.sample_rate(),
    )?;
    let mut buffer_data = Vec::with_capacity(buffer.length() as usize);
    for i in 0..buffer.length() {
        let v = sin(2.0 * 3.14 * (i as f64 / audio_ctx.sample_rate() as f64));
        buffer_data.push(v as f32);
    }
    buffer.copy_to_channel(&buffer_data, 0).expect("copy data");

    web_sys::console::debug_1(&JsValue::from_str(&format!(
        "buffer length: {}, v[10000]: {}",
        buffer.length(),
        buffer_data[10000]
    )));
    Ok(buffer)
}
