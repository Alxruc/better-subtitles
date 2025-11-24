use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use serde::Serialize;
use tauri_plugin_sql::{Migration, MigrationKind};

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
    let migrations = vec![  
        Migration {  
            version: 1,  
            description: "create transcripts table",  
            sql: "CREATE TABLE IF NOT EXISTS transcripts (  
                id INTEGER PRIMARY KEY AUTOINCREMENT,  
                url TEXT,
                title TEXT,
                duration INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",  
            kind: MigrationKind::Up,  
        },

        Migration {
        version: 2,
        description: "create segments table and index",
        sql: "
            CREATE TABLE IF NOT EXISTS segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                transcript_id INTEGER NOT NULL,
                start_time_sec INTEGER NOT NULL,
                end_time_sec INTEGER NOT NULL,
                text_content TEXT NOT NULL,
                FOREIGN KEY(transcript_id) REFERENCES transcripts(id)
            );
            
            -- Add an index for fast retrieval of segments by transcript ID
            CREATE INDEX idx_segments_transcript_id ON segments(transcript_id);
        ",
        kind: MigrationKind::Up,
    },
    ];  

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:subtitles.db", migrations)
                .build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![transcribe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
