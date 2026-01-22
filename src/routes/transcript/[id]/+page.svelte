<script>
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  const transcriptId = $page.params.id;
  let transcript = $state(null);
  let segments = $state([]);
  let loading = $state(true);

  onMount(() => {
    loadTranscriptData();
  });

  async function loadTranscriptData() {
    loading = true;
    try {
      // 1. Fetch Details
      transcript = await invoke("get_transcript_details", { id: transcriptId });
      
      // 2. Fetch Segments
      segments = await invoke("get_transcript_segments", { id: transcriptId });
      
    } catch (error) {
      console.error("Error loading transcript data:", error);
      goto("/");
    } finally {
      loading = false;
    }
  }

  async function deleteTranscript() {
    if(!confirm("Are you sure?")) return;
    
    try {
        await invoke("delete_transcript", { id: transcriptId });
        goto("/");
    } catch (error) {
        console.error("Error deleting transcript:", error);
        alert("Failed to delete: " + error);
    }
  }

  function formatTime(seconds) {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }

  function goBack() {
    goto("/");
  }

</script>

<main class="container">
  {#if loading}
    <div class="loading">Loading transcript...</div>
  {:else if transcript}
    <div class="header">
      <button class="back-button" onclick={goBack}>
        ← Back to Transcripts
      </button>
      <button class="delete-button" onclick={deleteTranscript()}>
        Delete Transcript
      </button>
      <h1>Transcript Details</h1>
      <div class="transcript-info">
        <p class="url">{transcript.url}</p>
        <p class="date">Created: {formatDate(transcript.created_at)}</p>
      </div>
    </div>

    <div class="segments-container">
      {#if segments.length === 0}
        <p class="empty-state">No segments found for this transcript.</p>
      {:else}
        {#each segments as segment, index (index)}
          <div class="segment">
            <div class="timestamp">
              {formatTime(segment.start)} - {formatTime(segment.end)}
            </div>
            <div class="text">{segment.text}</div>
          </div>
        {/each}
      {/if}
    </div>
  {:else}
    <div class="error">Transcript not found</div>
  {/if}
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
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  .loading,
  .error {
    text-align: center;
    padding: 3rem 1rem;
    font-size: 1.1rem;
    color: #666;
  }

  .header {
    margin-bottom: 2rem;
  }

  .back-button {
    background-color: transparent;
    border: 1px solid #ddd;
    color: #396cd8;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    margin-bottom: 1.5rem;
    transition: background-color 0.2s, border-color 0.2s, color 0.2s, transform 0.2s;
  }

  .back-button:hover {
    background-color: #f0f0f0;
    border-color: #396cd8;
    transform: translateY(-3px);
  }

  .delete-button {
    background-color: #d40d0d;
    border: 1px solid #ddd;
    color: #fff;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    margin-bottom: 1.5rem;
    transition: background-color 0.2s, border-color 0.2s, color 0.2s, transform 0.2s;
  }

  .delete-button:hover {
    background-color: #f0f0f0;
    border-color: #d40d0d;
    color: #1a1a1a;
    transform: translateY(-3px);
  }

  h1 {
    margin: 0 0 1rem 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .transcript-info {
    margin-top: 1rem;
  }

  .url {
    font-size: 1rem;
    margin: 0.5rem 0;
    word-break: break-all;
    color: #396cd8;
  }

  .date {
    font-size: 0.9rem;
    color: #666;
    margin: 0.5rem 0;
  }

  .segments-container {
    background-color: #ffffff;
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .segment {
    padding: 1rem;
    border-bottom: 1px solid #eee;
    display: grid;
    grid-template-columns: 140px 1fr;
    gap: 1rem;
    align-items: start;
  }

  .segment:last-child {
    border-bottom: none;
  }

  .timestamp {
    font-family: 'Courier New', monospace;
    font-size: 0.9rem;
    color: #666;
    font-weight: 500;
    white-space: nowrap;
  }

  .text {
    font-size: 1rem;
    line-height: 1.6;
    color: #0f0f0f;
  }

  .empty-state {
    text-align: center;
    color: #666;
    padding: 2rem;
  }

  @media (prefers-color-scheme: dark) {
    .back-button {
      border-color: #444;
      color: #6b9fff;
    }

    .back-button:hover {
      background-color: #3a3a3a;
      border-color: #6b9fff;
    }

    .segments-container {
      background-color: #1a1a1a;
    }

    .segment {
      border-bottom-color: #333;
    }

    .timestamp {
      color: #999;
    }

    .text {
      color: #f6f6f6;
    }

    .date {
      color: #999;
    }

    .empty-state {
      color: #999;
    }
  }

  @media (max-width: 640px) {
    .segment {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }

    .timestamp {
      font-size: 0.85rem;
    }
  }
</style>
