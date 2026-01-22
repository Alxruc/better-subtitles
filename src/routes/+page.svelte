<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from '@tauri-apps/api/event';
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { getTranscripts as fetchTranscripts } from "../lib/utils/transcription.js";


  let url = "";
  let transcriptions = [];
  let isTranscribing = false;

  onMount(async () => {
    getTranscripts();
    // Poll for updates every 2 seconds to catch new transcriptions
    const interval = setInterval(getTranscripts, 2000);

    const unlisten = await listen('deep-link://new-url', (event) => {
        const rawUrl = event.payload[0];
        console.log("Deep link received:", rawUrl);
        
        try {
          // Expected format: better-subtitles://save?url=https://youtube.com/...
          const urlObj = new URL(rawUrl);
          const action = urlObj.host; // "save"
          const targetUrl = urlObj.searchParams.get('url');

          if (action === 'save' && targetUrl) {
            console.log("Auto-starting transcription for:", targetUrl);
            
            // Update the UI input so the user sees what's happening
            url = targetUrl; 
            
            // Trigger the logic directly
            processTranscription(targetUrl);
          }
        } catch (e) {
          console.error("Failed to parse deep link:", e);
        }
    });


    return () => {
      clearInterval(interval);
      unlisten();
    }
  });

  async function getTranscripts() {
    transcriptions = await fetchTranscripts();
  }

  async function handleFormSubmit(event) {
    event.preventDefault();
    await processTranscription(url);
  }

  async function processTranscription(targetUrl) {
    if (!targetUrl.trim()) return;
    isTranscribing = true;
    
    try {
      // transcribing + saving to db
      const segments = await invoke("transcribe", { url: targetUrl });
      
      console.log("Transcription finished and saved!");
      
      url = ""; 
      
      // refresh the list from the DB
      await getTranscripts();
      
    } catch (error) {
      console.error("Error:", error);
      alert("Failed: " + error);
    } finally {
      isTranscribing = false;
    }
  }

  function viewTranscript(id) {
    goto(`/transcript/${id}`);
  }

  function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }

  function extractVideoId(url) {
    try {
      const urlObj = new URL(url);
      return urlObj.searchParams.get("v") || url.split("/").pop();
    } catch {
      return "";
    }
  }
</script>

<main class="container">
  <h1>Better Subtitle Generation for YouTube</h1>
  
  <form class="input-form" onsubmit={handleFormSubmit}>
    <input 
      id="url-input" 
      placeholder="Enter a YouTube URL..." 
      bind:value={url}
      disabled={isTranscribing}
    />
    <button type="submit" disabled={isTranscribing}>
      {isTranscribing ? "Transcribing..." : "Transcribe"}
    </button>
  </form>

  <div class="transcripts-grid">
    {#if transcriptions.length === 0}
      <p class="empty-state">No transcripts yet. Add a YouTube URL above to get started.</p>
    {:else}
      {#each transcriptions as transcript (transcript.id)}
        <button 
          class="transcript-card" 
          onclick={() => viewTranscript(transcript.id)}
        >
          <div class="card-thumbnail">
            <img 
              src={`https://img.youtube.com/vi/${extractVideoId(transcript.url)}/mqdefault.jpg`}
              alt="Video thumbnail"
              onerror={(e) => e.target.src = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="320" height="180" fill="%23666"%3E%3Crect width="320" height="180" fill="%23ddd"/%3E%3Ctext x="50%25" y="50%25" dominant-baseline="middle" text-anchor="middle" font-family="sans-serif" font-size="18" fill="%23666"%3ENo Preview%3C/text%3E%3C/svg%3E'}
            />
          </div>
          <div class="card-content">
            <h3 class="card-title">{transcript.url}</h3>
            <p class="card-date">{formatDate(transcript.created_at)}</p>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</main>

<style>
  :global(body) {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
    background-color: #f6f6f6;
    margin: 0;
    padding: 0;
  }

  @media (prefers-color-scheme: dark) {
    :global(body) {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  h1 {
    text-align: center;
    margin-bottom: 2rem;
    font-size: 2rem;
    font-weight: 600;
  }

  .input-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 3rem;
    max-width: 600px;
    margin-left: auto;
    margin-right: auto;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    transition: border-color 0.25s;
  }

  input {
    flex: 1;
    color: #0f0f0f;
    background-color: #ffffff;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    outline: none;
  }

  button {
    cursor: pointer;
    color: #ffffff;
    background-color: #396cd8;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button:hover:not(:disabled) {
    background-color: #2d5ab8;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .transcripts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.5rem;
  }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    color: #666;
    padding: 3rem 1rem;
    font-size: 1.1rem;
  }

  .transcript-card {
    background-color: #ffffff;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    transition: transform 0.2s, box-shadow 0.2s;
    cursor: pointer;
    border: 1px solid transparent;
    padding: 0;
    text-align: left;
    width: 100%;
  }

  .transcript-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    border-color: #396cd8;
  }

  .card-thumbnail {
    width: 100%;
    aspect-ratio: 16 / 9;
    background-color: #e0e0e0;
    overflow: hidden;
  }

  .card-thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-content {
    padding: 1rem;
  }

  .card-title {
    font-size: 0.95rem;
    font-weight: 500;
    margin: 0 0 0.5rem 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #0f0f0f;
  }

  .card-date {
    font-size: 0.85rem;
    color: #666;
  }
</style>