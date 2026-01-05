import Database from "@tauri-apps/plugin-sql";

export async function getTranscripts() {
  try {
    const db = await Database.load("sqlite:subtitles.db");
    const result = await db.select("SELECT * FROM transcripts ORDER BY created_at DESC");
    return result.map(tr => ({
      id: tr.id,
      url: tr.url,
      duration: tr.duration,
      created_at: tr.created_at,
    }));
  } catch (error) {
    console.log("[v0] Error loading transcripts:", error);
    return [];
  }
}

export async function setTranscript(transcript) {
  try {
    const db = await Database.load("sqlite:subtitles.db");
    const result = await db.execute(
      "INSERT INTO transcripts (url, duration, created_at) VALUES ($1, $2, $3)", 
      [transcript.url, transcript.duration, transcript.created_at]
    );
    return result.lastInsertId;
  } catch (error) {
    console.log("[v0] Error saving transcript:", error);
    throw error;
  }
}

export async function setSegment(segment) {
  try {
    const db = await Database.load("sqlite:subtitles.db");
    await db.execute(
      "INSERT INTO segments (transcript_id, start_time_sec, end_time_sec, text_content) VALUES ($1, $2, $3, $4)", 
      [segment.transcript_id, segment.start_time_sec, segment.end_time_sec, segment.text_content]
    );
  } catch (error) {
    console.log("[v0] Error saving segment:", error);
    throw error;
  }
}