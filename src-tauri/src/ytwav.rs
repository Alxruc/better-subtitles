use std::io::Read;
use std::process::{Command, Stdio};

pub async fn youtube_to_pcm_i16(url: &str) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    // yt-dlp bestaudio to stdout
    let mut yt = Command::new("yt-dlp")
        .arg("-f")
        .arg("bestaudio")
        .arg("-o")
        .arg("-")
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()?;

    let yt_stdout = yt.stdout.take().unwrap();

    // ffmpeg -> raw PCM s16le (mono, 16kHz)
    let mut ffmpeg = Command::new("ffmpeg")
        .arg("-i")
        .arg("pipe:0")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-f")
        .arg("s16le")
        .arg("pipe:1")
        .stdin(Stdio::from(yt_stdout))
        .stdout(Stdio::piped())
        .spawn()?;

    let mut raw_bytes = Vec::new();
    ffmpeg.stdout.take().unwrap().read_to_end(&mut raw_bytes)?;

    // Wait for processes to complete
    let ffmpeg_status = ffmpeg.wait()?;
    let yt_status = yt.wait()?;

    if !ffmpeg_status.success() {
        return Err("ffmpeg failed".into());
    }
    if !yt_status.success() {
        return Err("yt-dlp failed".into());
    }

    // Convert bytes to i16
    let pcm: Vec<i16> = raw_bytes
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(pcm)
}
