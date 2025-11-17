use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use serde::Serialize;

mod ytwav;

const MODEL_PATH: &str = "../models/ggml-kotoba-whisper-v2.0.bin";

#[derive(Serialize)]
pub struct TranscriptionSegment {
    pub start: f32, // seconds
    pub end: f32,   // seconds
    pub text: String,
}

#[tauri::command]
fn transcribe(url: &str) -> Vec<TranscriptionSegment> {
    let ctx = WhisperContext::new_with_params(
        MODEL_PATH,
        WhisperContextParameters::default()
    ).expect("failed to load model");

    // create a params object
    let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    let st = std::time::Instant::now();
    let audio_i16 = ytwav::youtube_to_pcm_i16(url).expect("Error in converting URL to WAV");

    //  Convert to f32 for whisper
    let mut float_samples = vec![0.0f32; audio_i16.len()];

    whisper_rs::convert_integer_to_float_audio(
        &audio_i16,
        &mut float_samples,
    ).expect("Audio conversion failed");

    let et = std::time::Instant::now();
    println!(
        "-> Loaded and converted audio file (took {}ms)",
        (et - st).as_millis()
    );

    let st = std::time::Instant::now();
    let mut state = ctx.create_state().expect("failed to create state");
    state
        .full(params, &float_samples[..])
        .expect("failed to run model");

     let segments: Vec<TranscriptionSegment> = state
        .as_iter()
        .map(|segment| TranscriptionSegment {
            start: segment.start_timestamp() as f32 / 100.0,
            end: segment.end_timestamp() as f32 / 100.0,
            text: segment.to_string(),
        })
        .collect();

    
    let et = std::time::Instant::now();
    println!("-> Finished (took {}ms)", (et - st).as_millis());

    return segments
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![transcribe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
