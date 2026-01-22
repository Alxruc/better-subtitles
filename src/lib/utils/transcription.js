import { invoke } from "@tauri-apps/api/core";

export async function getTranscripts() {
  try {
    // Direct call to Rust
    return await invoke("get_transcripts");
  } catch (error) {
    console.error("Error loading transcripts:", error);
    return [];
  }
}
