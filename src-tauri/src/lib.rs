use serde::{Deserialize,Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri::{Manager,Emitter, State};
use axum::{
    extract::{Query, State as AxumState},
    routing::get,
    Router,
    Json,
};
use sqlx::sqlite::{SqliteConnectOptions,SqliteJournalMode};
use sqlx::{FromRow, SqlitePool};
use tower_http::cors::{CorsLayer, Any};
use serde_json::{json, Value};
use std::net::SocketAddr;


mod ytwav;

const MODEL_NAME: &str = "ggml-base.bin";
type DbPool = SqlitePool;

#[derive(Serialize, FromRow)]
pub struct TranscriptionSegment {
    pub start: f32, // seconds
    pub end: f32,   // seconds
    pub text: String,
}

#[derive(Serialize, FromRow)]
pub struct Transcript {
    pub id: i64,
    pub url: String,
    pub duration: i64,
    pub created_at: String,
}

#[derive(Deserialize)]
struct SubtitleParams {
    url: String,
}

#[tauri::command]
async fn transcribe(app_handle: tauri::AppHandle, pool: State<'_, DbPool>, url: &str) -> Result<Vec<TranscriptionSegment>, String> {
    let resource_dir = app_handle
        .path()
        .resolve("resources/models", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;

    let model_buf = resource_dir.join(MODEL_NAME);

    // borrow the string slice from it
    let model_path = model_buf
        .to_str()
        .expect("Path contains invalid UTF-8 characters");

    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .expect("failed to load model");

    // create a params object
    let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    let st = std::time::Instant::now();
    let audio_i16 = ytwav::youtube_to_pcm_i16(url)
        .await
        .expect("Error in converting URL to WAV");

    //  Convert to f32 for whisper
    let mut float_samples = vec![0.0f32; audio_i16.len()];

    whisper_rs::convert_integer_to_float_audio(&audio_i16, &mut float_samples)
        .expect("Audio conversion failed");

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

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let transcript_id: i64 = sqlx::query_scalar(
        "INSERT INTO transcripts (url, duration) VALUES (?, ?) RETURNING id"
    )
    .bind(url)
    .bind(0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // We iterate and insert should probably build into one query, good enough
    for segment in &segments {
        sqlx::query(
            "INSERT INTO segments (transcript_id, start_time_sec, end_time_sec, text_content) VALUES (?, ?, ?, ?)"
        )
        .bind(transcript_id)
        .bind(segment.start) // Ensure your DB column is REAL, or cast to int if you kept it INTEGER
        .bind(segment.end)
        .bind(&segment.text)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Commit the transaction
    tx.commit().await.map_err(|e| e.to_string())?;

    println!("Saved transcript {} with {} segments.", transcript_id, segments.len());

    let et = std::time::Instant::now();
    println!("-> Finished (took {}ms)", (et - st).as_millis());

    return Ok(segments);
}

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Transcripts Table
    sqlx::query("CREATE TABLE IF NOT EXISTS transcripts (  
        id INTEGER PRIMARY KEY AUTOINCREMENT,  
        url TEXT,
        duration INTEGER NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )").execute(pool).await?;

    // Segments Table
    sqlx::query("CREATE TABLE IF NOT EXISTS segments (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        transcript_id INTEGER NOT NULL,
        start_time_sec REAL NOT NULL,
        end_time_sec REAL NOT NULL,
        text_content TEXT NOT NULL,
        FOREIGN KEY(transcript_id) REFERENCES transcripts(id)
    )").execute(pool).await?;

    // Index
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_segments_transcript_id ON segments(transcript_id)")
        .execute(pool).await?;

    Ok(())
}

#[tauri::command]
async fn get_transcripts(pool: State<'_, DbPool>) -> Result<Vec<Transcript>, String> {
    let transcripts = sqlx::query_as::<_, Transcript>(
        "SELECT id, url, duration, created_at FROM transcripts ORDER BY created_at DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(transcripts)
}

#[tauri::command]
async fn get_transcript_details(pool: State<'_, DbPool>, id: i64) -> Result<Transcript, String> {
    let transcript = sqlx::query_as::<_, Transcript>(
        "SELECT id, url, duration, created_at FROM transcripts WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Transcript not found".to_string())?;

    Ok(transcript)
}

#[tauri::command]
async fn get_transcript_segments(pool: State<'_, DbPool>, id: i64) -> Result<Vec<TranscriptionSegment>, String> {
    let segments = sqlx::query_as::<_, TranscriptionSegment>(
        "SELECT start_time_sec AS start, end_time_sec AS end, text_content AS text 
         FROM segments WHERE transcript_id = ? ORDER BY start_time_sec ASC"
    )
    .bind(id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(segments)
}

#[tauri::command]
async fn delete_transcript(pool: State<'_, DbPool>, id: i64) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM segments WHERE transcript_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM transcripts WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn get_subtitles(
    AxumState(pool): AxumState<SqlitePool>,
    Query(params): Query<SubtitleParams>,
) -> Json<Value> {
    println!("Fetching subtitles for URL: {}", params.url);
    //  Find the transcript ID for the given URL
    let transcript_id: Option<i64> = sqlx::query_scalar("SELECT id FROM transcripts WHERE url = ?")
        .bind(&params.url)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    let id = match transcript_id {
        Some(i) => i,
        None => {
            return Json(json!({ 
                "error": "No transcript found", 
                "url": params.url 
            }));
        }
    };

    //  Get segments for that ID
    let segments_result = sqlx::query_as::<_, TranscriptionSegment>(
    "SELECT 
        start_time_sec AS start, 
        end_time_sec AS end, 
        text_content AS text 
     FROM segments 
     WHERE transcript_id = ? 
     ORDER BY start_time_sec ASC"
    )
    .bind(id)
    .fetch_all(&pool)
    .await;

    match segments_result {
        Ok(segments) => {
            // Map to the JSON format external app expects
            let response_data: Vec<Value> = segments
                .into_iter()
                .map(|s| {
                    json!({
                        "start": s.start,
                        "text": s.text
                    })
                })
                .collect();
            Json(json!(response_data))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn start_server(pool: SqlitePool) {
    let app = Router::new()
        .route("/subtitles", get(get_subtitles))
        .with_state(pool) // Pass DB pool to all routes
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

    let addr = SocketAddr::from(([127, 0, 0, 1], 14567));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            println!("New instance attempt detected: {:?}", argv);

            // Focus the main window
            let window = app.get_webview_window("main").unwrap();
            window.set_focus().unwrap();

            //  Find the URL in the arguments
            // argv is a list of strings. On Windows, the URL is usually one of them.
            // We look for the one starting with your scheme.
            let url_arg = argv.iter().find(|arg| arg.starts_with("better-subtitles://"));

            if let Some(url) = url_arg {
                // Manually emit the event to Svelte
                let _ = app.emit("deep-link://new-url", vec![url]);
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            // We use "if let Err" to print a warning instead of crashing
            if let Err(e) = app.deep_link().register("better-subtitles") {
                println!("Warning: Deep link registration failed: {}", e);
            };
            //Resolve Path
            let app_data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            let db_path = app_data_dir.join("better-subtitles.db");

            // initialize DB & run migrations (blocking)
            let pool = tauri::async_runtime::block_on(async move {
                let options = SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal);

                let pool = SqlitePool::connect_with(options).await.unwrap();
                init_db(&pool).await.expect("Database migration failed");
                
                pool
            });

            app.manage(pool.clone());
           
            tauri::async_runtime::spawn(async move {
               start_server(pool).await; 
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            transcribe, 
            get_transcripts, 
            get_transcript_details, 
            get_transcript_segments, 
            delete_transcript
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
