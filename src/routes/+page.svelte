<script>
  import { invoke } from "@tauri-apps/api/core";
  import Database from "@tauri-apps/plugin-sql";

  let url = "";
  let transcription = [];
  let transcriptionSegments = [];
  let done = false;

  async function getTranscripts() {
    try {
      const db = await Database.load("sqlite:subtitles.db");
      const result = await db.select("SELECT * FROM transcripts");
      transcription = result.map(tr => ({
        id: tr.id,
        url: tr.url,
        duration: tr.duration,
        created_at: tr.created_at,
      }));
    } catch (error) {
      console.log(error);
    }
  }

  async function setTranscript(transcript) {
    try {
      const db = await Database.load("sqlite:subtitles.db");
      let result = await db.execute("INSERT INTO transcripts (url, duration, created_at) VALUES ($1, $2, $3)", [
        transcript.url,
        transcript.duration,
        transcript.created_at
      ]);
      getTranscripts();
      return result.lastInsertId;
    } catch (error) {
      console.log(error);
    }
  }

  async function getSegments() {
    try {
      const db = await Database.load("sqlite:subtitles.db");
      const result = await db.select("SELECT * FROM segments");
      transcriptionSegments = result.map(seg => ({
        start: seg.start_time_sec,
        end: seg.end_time_sec,
        text: seg.text_content
      }));
    } catch (error) {
      console.log(error);
    }
  }

  async function setSegment(segment) {
    try {
      const db = await Database.load("sqlite:subtitles.db");
      await db.execute("INSERT INTO segments (transcript_id, start_time_sec, end_time_sec, text_content) VALUES ($1, $2, $3, $4)", [
        segment.transcript_id,
        segment.start_time_sec,
        segment.end_time_sec,
        segment.text_content,
      ]);
      getSegments();
    } catch (error) {
      console.log(error);
    }
  }

  async function startTranscription(event) {
    event.preventDefault();
    transcriptionSegments = await invoke("transcribe", { url });
    const transcriptId = await setTranscript({ url, duration: 0, created_at: new Date().toISOString() });

    for (const segment of transcriptionSegments) {
      await setSegment({
        transcript_id: transcriptId,
        start_time_sec: segment.start,
        end_time_sec: segment.end,
        text_content: segment.text,
      });
    }

    done = true;
  }
</script>

<main class="container">
  <h1> Better Subtitle Generation for YouTube </h1>

  <form class="row" onsubmit={startTranscription}>
    <input id="greet-input" placeholder="Enter a YouTube URL..." bind:value={url} />
    <button type="submit">Transcribe</button>
  </form>
  {#if done }
    <ul>
    {#each transcriptionSegments as segment}
      <li>[{segment.start} - {segment.end}] {segment.text}</li>
    {/each}
    </ul>
  {/if}
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
}


h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }


  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
