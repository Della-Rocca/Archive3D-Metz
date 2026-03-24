<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { authStore } from "$lib/stores/auth";

  interface AppConfig {
    depot_path: string;
    validation_path: string;
    archive_path: string;
    settings_path: string;
    admin_password?: string;
  }

  let config: AppConfig = {
    depot_path: "",
    validation_path: "",
    archive_path: "",
    settings_path: "",
  };

  let step = 0;
  let saving = false;
  let error = "";

  const stepsInfo = [
    {
      title: "Bienvenue dans Archive3D-Metz",
      desc: "Pour commencer, nous devons configurer les chemins des dossiers principaux sur votre ordinateur. Cela ne prendra qu'une minute."
    },
    {
      field: "depot_path" as const,
      title: "Dossier Dépôt",
      desc: "Ce dossier servira à réceptionner toutes les structures déposées, en attente de validation."
    },
    {
      field: "validation_path" as const,
      title: "Dossier Validation",
      desc: "Zone de travail temporaire où l'administrateur peut modifier, vérifier, générer les modèles 3D, etc."
    },
    {
      field: "archive_path" as const,
      title: "Dossier Archive",
      desc: "Le lieu de stockage définitif sécurisé. C'est ici que sont consultables les modèles terminés."
    },
    {
      field: "settings_path" as const,
      title: "Dossier Paramètres (Presets)",
      desc: "Configuration de l'équipe (auteurs, etc) et logs d'audit. Tous les fichiers seront générés automatiquement ici."
    },
    {
      title: "C'est terminé !",
      desc: "La configuration est prête. Cliquons sur Enregistrer pour créer la configuration de l'application."
    }
  ];

  $: currentField = stepsInfo[step].field as "depot_path" | "validation_path" | "archive_path" | "settings_path" | undefined;
  $: isCurrentFieldFilled = currentField ? !!config[currentField] : false;

  onMount(async () => {
    // Vérification de sécurité
    if ($authStore.role !== "admin") {
      goto("/login");
      return;
    }
    
    try {
      const existing = await invoke<AppConfig>("get_app_config");
      config = { ...existing };
    } catch (e) {
      console.warn("Could not load initial config or is default", e);
    }
  });

  async function pickPath(field: "depot_path" | "validation_path" | "archive_path" | "settings_path") {
    const result = await open({
      directory: true,
      title: `Sélectionnez le dossier pour ${field}`
    });
    if (result) {
      config[field] = result as string;
    }
  }

  function nextStep() {
    error = "";
    const currentInfo = stepsInfo[step];
    
    if (currentInfo.field) {
      if (!config[currentInfo.field as keyof AppConfig]) {
        error = "Vous devez sélectionner un dossier pour continuer.";
        return;
      }
    }
    
    if (step < stepsInfo.length - 1) {
      step++;
    }
  }

  function prevStep() {
    error = "";
    if (step > 0) step--;
  }

  async function saveConfig() {
    saving = true;
    error = "";
    try {
      await invoke("update_app_config", { newConfig: config });
      await invoke("ensure_default_files").catch(() => {});
      goto("/depot");
    } catch (e) {
      error = `Erreur de sauvegarde: ${e}`;
      saving = false;
    }
  }
</script>

<div class="setup-container">
  <main class="setup-card">
    <div class="setup-progress">
      <div 
        class="progress-bar" 
        style="width: {(step / (stepsInfo.length - 1)) * 100}%"
      ></div>
    </div>
    
    <header class="setup-header">
      <div class="setup-icon">
        {#if step === 0}
          <svg viewBox="0 0 24 24" fill="none" width="32" height="32"><path d="M5 12L10 17L20 7" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        {:else if step === stepsInfo.length - 1}
          <svg viewBox="0 0 24 24" fill="none" width="32" height="32"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/><path d="M8 12.5L11 15.5L16 9.5" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" width="32" height="32"><path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        {/if}
      </div>
      <h1>{stepsInfo[step].title}</h1>
      <p>{stepsInfo[step].desc}</p>
    </header>

    <div class="setup-body">
      {#if currentField}
        <div class="path-selection" class:path-selection-valid={isCurrentFieldFilled}>
          <label class="meta-label" for="path-input">Chemin actuel</label>
          <div class="path-row">
            <input 
              id="path-input"
              class="meta-input" 
              readonly
              value={config[currentField]}
              placeholder="Aucun dossier sélectionné..."
            />
            <button class="btn btn-secondary browse-btn" on:click={() => { if(currentField) pickPath(currentField); }}>
              Parcourir...
            </button>
          </div>
        </div>

        {#if isCurrentFieldFilled}
          <div class="setup-success-msg slide-up">
            <svg viewBox="0 0 24 24" fill="none" width="16" height="16"><path d="M5 13L9 17L19 7" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
            Dossier configuré, vous pouvez passer à l'étape suivante.
          </div>
        {/if}
      {/if}

      {#if error}
        <div class="status-message error">{error}</div>
      {/if}
    </div>

    <footer class="setup-footer">
      {#if step > 0}
        <button class="btn btn-secondary" on:click={prevStep} disabled={saving}>
          Retour
        </button>
      {:else}
        <div></div> <!-- spacer -->
      {/if}

      {#if step === stepsInfo.length - 1}
        <button class="btn btn-primary pulse-btn" on:click={saveConfig} disabled={saving}>
          {saving ? 'Enregistrement...' : 'Terminer la configuration'}
        </button>
      {:else}
        <button class="btn btn-primary" class:pulse-btn={isCurrentFieldFilled} on:click={nextStep}>
          Suivant
        </button>
      {/if}
    </footer>
  </main>
</div>

<style>
  .setup-container {
    min-height: 100dvh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-lg);
    background:
      radial-gradient(1000px 600px at 50% 0%, rgba(30, 58, 95, 0.1), transparent 50%),
      linear-gradient(180deg, #f8fafc 0%, #edf2f8 100%);
  }

  .setup-card {
    width: 100%;
    max-width: 640px;
    background: #ffffff;
    border-radius: 20px;
    box-shadow: 0 20px 40px rgba(15, 23, 42, 0.08);
    overflow: hidden;
    position: relative;
    border: 1px solid rgba(148, 163, 184, 0.2);
  }

  .setup-progress {
    height: 4px;
    background: var(--color-neutral-300);
    width: 100%;
  }

  .progress-bar {
    height: 100%;
    background: var(--color-primary);
    transition: width 0.4s cubic-bezier(0.2, 0, 0, 1);
  }

  .setup-header {
    text-align: center;
    padding: var(--spacing-2xl) var(--spacing-xl) var(--spacing-lg);
  }

  .setup-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: linear-gradient(135deg, rgba(30, 58, 95, 0.1) 0%, rgba(63, 106, 162, 0.05) 100%);
    color: var(--color-primary);
    margin-bottom: var(--spacing-md);
  }

  .setup-header h1 {
    margin: 0 0 var(--spacing-sm);
    font-size: 1.75rem;
    color: var(--color-neutral-900);
    font-weight: 800;
  }

  .setup-header p {
    margin: 0 auto;
    max-width: 48ch;
    color: var(--color-neutral-600);
    font-size: 0.95rem;
    line-height: 1.5;
  }

  .setup-body {
    padding: 0 var(--spacing-2xl) var(--spacing-xl);
    min-height: 120px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: var(--spacing-md);
  }

  .path-selection {
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-lg);
    border: 1px dashed var(--color-neutral-400);
    transition: all 0.3s ease;
  }

  .path-selection-valid {
    background: rgba(46, 138, 78, 0.04);
    border: 1px solid rgba(46, 138, 78, 0.3);
  }

  .setup-success-msg {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--color-success);
    font-size: 0.85rem;
    font-weight: 600;
    margin-top: -4px;
    padding: 4px 8px;
  }

  .slide-up {
    animation: slideUp 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .pulse-btn {
    animation: pulseGlow 2s infinite cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes pulseGlow {
    0% { box-shadow: 0 0 0 0 rgba(30, 58, 95, 0); }
    30% { box-shadow: 0 0 0 6px rgba(30, 58, 95, 0.2); }
    100% { box-shadow: 0 0 0 10px rgba(30, 58, 95, 0); }
  }

  .path-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--spacing-sm);
    margin-top: 4px;
  }

  .browse-btn {
    white-space: nowrap;
  }

  .setup-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-lg) var(--spacing-2xl);
    background: var(--color-neutral-200);
    border-top: 1px solid var(--color-neutral-300);
  }

  @media (max-width: 600px) {
    .setup-body {
      padding: 0 var(--spacing-lg) var(--spacing-xl);
    }
    .setup-footer {
      padding: var(--spacing-lg);
    }
    .path-row {
      grid-template-columns: 1fr;
    }
  }
</style>
