<script module lang="ts">
  // Garde au niveau module : une seule vérification par session de l'application,
  // même si le layout est remonté (HMR, navigation).
  let hasChecked = false;
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  let update: Update | null = $state(null);
  let visible = $state(false);
  let installing = $state(false);
  let errorMsg = $state("");
  let contentLength = $state(0);
  let downloaded = $state(0);

  let progressPct = $derived(
    contentLength > 0 ? Math.min(100, Math.round((downloaded / contentLength) * 100)) : 0
  );

  onMount(async () => {
    if (hasChecked) return;
    hasChecked = true;
    try {
      const result = await check();
      if (result) {
        update = result;
        visible = true;
      }
    } catch (e) {
      // Hors ligne ou endpoint indisponible : échec silencieux, on ne bloque pas le démarrage
      console.debug("Vérification de mise à jour impossible :", e);
    }
  });

  async function installNow() {
    if (!update || installing) return;
    installing = true;
    errorMsg = "";
    try {
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            downloaded = 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            break;
          case "Finished":
            downloaded = contentLength;
            break;
        }
      });
      // Sous Windows l'installateur ferme l'application lui-même ;
      // relaunch() couvre macOS et Linux.
      await relaunch();
    } catch (e) {
      installing = false;
      errorMsg = "La mise à jour a échoué. Veuillez réessayer plus tard.";
      console.error("Échec de la mise à jour :", e);
    }
  }

  function later() {
    if (installing) return;
    visible = false;
  }
</script>

{#if visible && update}
  <div class="update-overlay" role="dialog" aria-modal="true" aria-labelledby="update-title">
    <div class="update-modal">
      <div class="update-header">
        <h2 id="update-title">Nouvelle version disponible</h2>
        <p class="update-subtitle">
          Version {update.version} — vous utilisez actuellement la version {update.currentVersion}
        </p>
      </div>
      <div class="update-body">
        {#if update.body}
          <div class="release-notes">{update.body}</div>
        {/if}
        {#if installing}
          <div class="progress-track">
            <div class="progress-fill" style:width={`${progressPct}%`}></div>
          </div>
          <p class="progress-label">
            {contentLength > 0 ? `Téléchargement… ${progressPct}%` : "Téléchargement…"}
          </p>
        {/if}
        {#if errorMsg}
          <p class="status-message error">{errorMsg}</p>
        {/if}
      </div>
      <div class="update-actions">
        <button type="button" class="btn btn-secondary" on:click={later} disabled={installing}>
          Plus tard
        </button>
        <button type="button" class="btn btn-primary" on:click={installNow} disabled={installing}>
          {installing ? "Installation…" : "Mettre à jour maintenant"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .update-overlay {
    position: fixed;
    inset: 0;
    z-index: 2000;
    background: rgba(15, 23, 42, 0.42);
    backdrop-filter: blur(5px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
  }

  .update-modal {
    width: 100%;
    max-width: 480px;
    background: var(--color-neutral-100);
    border-radius: var(--border-radius-lg);
    border: var(--border-width) solid var(--color-neutral-400);
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.24);
    padding: var(--spacing-2xl);
    font-family: var(--font-family);
    box-sizing: border-box;
  }

  .update-header h2 {
    margin: 0;
    font-size: var(--font-size-xl);
    color: var(--color-primary);
    letter-spacing: -0.02em;
  }

  .update-subtitle {
    margin: var(--spacing-sm) 0 0;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
  }

  .update-body {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin: var(--spacing-xl) 0;
  }

  .release-notes {
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    background: var(--color-neutral-200);
    border: var(--border-width) solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
  }

  .progress-track {
    height: 8px;
    background: var(--color-neutral-400);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary);
    border-radius: 999px;
    transition: width 0.2s ease;
  }

  .progress-label {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
    text-align: center;
  }

  .update-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-md);
  }
</style>
