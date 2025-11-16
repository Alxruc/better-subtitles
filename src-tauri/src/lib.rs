// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

const MODEL_PATH: &str = "../models/ggml-kotoba-whisper-v2.0.bin";
const TEMP_EXAMPLE: &str = "../example.wav";

#[tauri::command]
fn greet(name: &str) -> String {
    println!("hello {}", name);
    let ctx = WhisperContext::new_with_params(
        MODEL_PATH,
        WhisperContextParameters::default()
    ).expect("failed to load model");

    // create a params object
    let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    let st = std::time::Instant::now();
    let mut reader = hound::WavReader::open(TEMP_EXAMPLE).unwrap();

    let int_samples: Vec<i16> = reader
        .samples::<i16>()
        .map(|s| s.expect("invalid sample"))
        .collect();

    //  Convert to f32 for whisper
    let mut float_samples = vec![0.0f32; int_samples.len()];

    whisper_rs::convert_integer_to_float_audio(
        &int_samples,
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

    for segment in state.as_iter() {
        println!(
            "[{} - {}]: {}",
            // note start and end timestamps are in centiseconds
            // (10s of milliseconds)
            segment.start_timestamp(),
            segment.end_timestamp(),
            // the Display impl for WhisperSegment will replace invalid UTF-8 with the Unicode replacement character
            segment
        );
    }
    let et = std::time::Instant::now();

    return format!("-> Finished (took {}ms)", (et - st).as_millis());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
