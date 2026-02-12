<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  let { selectedModel = $bindable("") } = $props();

  let models: string[] = $state([]);
  let isLoading = $state(false);

  async function loadModels() {
    try {
      models = await invoke("get_available_models");
      // Default to first available if none selected
      if (!selectedModel && models.length > 0) {
        selectedModel = models[0];
      }
    } catch (e) {
      console.error("Failed to load models", e);
    }
  }

  async function handleImport() {
    try {
      // Open native file dialog
      const selected = await open({
        multiple: false,
        filters: [{ name: "Whisper Models", extensions: ["bin"] }],
      });

      if (selected) {
        isLoading = true;
        // Copy file to internal app storage
        const filename = await invoke("import_model", { filePath: selected });
        await loadModels(); // Refresh list
        selectedModel = filename as string; // Auto-select new model
        isLoading = false;
      }
    } catch (e) {
      console.error("Import failed", e);
      isLoading = false;
    }
  }

  onMount(loadModels);
</script>

<div class="selector-wrapper">
  <select bind:value={selectedModel} disabled={models.length === 0}>
    {#each models as model}
      <option value={model}>{model}</option>
    {/each}
    {#if models.length === 0}
      <option disabled>No models found</option>
    {/if}
  </select>

  <button onclick={handleImport} disabled={isLoading}>
    {isLoading ? 'Importing...' : 'Add Model'}
  </button>
</div>