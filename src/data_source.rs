use js_sys::Math::sin;
use wasm_bindgen::JsValue;
use web_sys::{AudioBuffer, AudioContext};

pub fn sine_buffer(audio_ctx: &AudioContext) -> Result<AudioBuffer, JsValue> {
    // let sample_rate = audio_ctx.sample_rate();
    let sample_rate = 96000.0;
    let buffer = audio_ctx.create_buffer(1, (sample_rate * 1.0) as u32, sample_rate)?;
    let mut buffer_data = Vec::with_capacity(buffer.length() as usize);
    for i in 0..buffer.length() {
        let v = sin(2.0 * 3.14 * (i as f64 / sample_rate as f64 * 10.0));
        buffer_data.push(v as f32);
    }
    buffer.copy_to_channel(&buffer_data, 0).expect("copy data");

    Ok(buffer)
}
pub fn line_buffer(audio_ctx: &AudioContext) -> Result<AudioBuffer, JsValue> {
    let buffer = audio_ctx.create_buffer(
        1,
        (audio_ctx.sample_rate() * 1.0) as u32,
        audio_ctx.sample_rate(),
    )?;
    let mut buffer_data = Vec::with_capacity(buffer.length() as usize);
    let mut flag = 1.0;
    for i in 0..buffer.length() {
        if i % 1000 == 0 {
            flag = -flag;
        }
        buffer_data.push(flag * 0.5f32);
    }
    buffer.copy_to_channel(&buffer_data, 0).expect("copy data");

    Ok(buffer)
}
