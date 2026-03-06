<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onDestroy, onMount } from "svelte";

  interface AppConfig {
    depot_path: string;
    validation_path: string;
    archive_path: string;
    presets_path: string;
    logs_path: string;
    revision_tags_path: string;
    admin_password: string;
  }

  interface PathValidationResult {
    valid: boolean;
    errors: string[];
    warnings: string[];
  }

  let config: AppConfig = {
    depot_path: "",
    validation_path: "",
    archive_path: "",
    presets_path: "",
    logs_path: "",
    revision_tags_path: "",
    admin_password: "",
  };

  type PathField = "depot_path" | "validation_path" | "archive_path" | "presets_path" | "logs_path" | "revision_tags_path";
  type PathFieldMeta = {
    field: PathField;
    label: string;
    hint: string;
  };

  // Champs qui utilisent un sélecteur de fichier (et non de dossier)
  const filePickerFields: PathField[] = ["presets_path", "logs_path", "revision_tags_path"];

  const pathFieldMeta: PathFieldMeta[] = [
    {
      field: "depot_path",
      label: "Dossier Dépôt",
      hint: "Source des structures déposées avant validation.",
    },
    {
      field: "validation_path",
      label: "Dossier Validation",
      hint: "Zone de contrôle qualité avant archivage.",
    },
    {
      field: "archive_path",
      label: "Dossier Archive",
      hint: "Stockage définitif des structures archivées.",
    },
    {
      field: "presets_path",
      label: "Fichier metadata-presets.json",
      hint: "Référentiel des listes de métadonnées. Créé automatiquement avec des listes vides si le fichier est absent.",
    },
    {
      field: "logs_path",
      label: "Fichier audit.log",
      hint: "Journal de traçabilité. Créé automatiquement à la première action si le fichier est absent.",
    },
    {
      field: "revision_tags_path",
      label: "Fichier revision-tags.json",
      hint: "Marques de révision pour les structures. Créé automatiquement avec un objet vide si le fichier est absent.",
    },
  ];

  let pathValidation: PathValidationResult | null = null;
  let saving = false;
  let status = "";
  let statusType: "success" | "error" | "" = "";
  // Snapshot de la config sauvegardée (pour afficher l'ancien chemin en cas de modification)
  let savedConfig: AppConfig | null = null;
  let validationDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let readyForLiveValidation = false;
  type FieldIssues = Record<PathField, { errors: string[]; warnings: string[] }>;
  const emptyFieldIssues = (): FieldIssues => ({
    depot_path: { errors: [], warnings: [] },
    validation_path: { errors: [], warnings: [] },
    archive_path: { errors: [], warnings: [] },
    presets_path: { errors: [], warnings: [] },
    logs_path: { errors: [], warnings: [] },
    revision_tags_path: { errors: [], warnings: [] },
  });

  const fieldMessageMatchers: Record<PathField, string[]> = {
    depot_path: ["dépôt", "depot"],
    validation_path: ["validation"],
    archive_path: ["archive"],
    presets_path: ["metadata-presets", "presets"],
    logs_path: ["logs", "audit"],
    revision_tags_path: ["revision-tags", "revision_tags"],
  };

  function mapMessageToField(message: string): PathField | null {
    const lower = message.toLowerCase();
    for (const [field, tokens] of Object.entries(fieldMessageMatchers) as [PathField, string[]][]) {
      if (tokens.some((token) => lower.includes(token))) {
        return field;
      }
    }
    return null;
  }

  function computeFieldIssues(validation: PathValidationResult | null): FieldIssues {
    const issues = emptyFieldIssues();
    if (!validation) return issues;

    for (const err of validation.errors) {
      const field = mapMessageToField(err);
      if (field) issues[field].errors.push(err);
    }

    for (const warn of validation.warnings) {
      const field = mapMessageToField(warn);
      if (field) issues[field].warnings.push(warn);
    }

    return issues;
  }

  $: pathFieldIssues = computeFieldIssues(pathValidation);
  $: globalValidationErrors = (pathValidation?.errors ?? []).filter((message) => !mapMessageToField(message));
  $: globalValidationWarnings = (pathValidation?.warnings ?? []).filter((message) => !mapMessageToField(message));

  function isPathFieldValid(field: PathField): boolean {
    const hasValue = config[field].trim().length > 0;
    if (!hasValue) return false;
    // Les avertissements (ex: "sera créé") ne bloquent pas le compteur
    return pathFieldIssues[field].errors.length === 0;
  }

  $: validationErrorCount = pathValidation?.errors.length ?? 0;
  $: validationWarningCount = pathValidation?.warnings.length ?? 0;
  $: validationIsHealthy = validationErrorCount === 0 && validationWarningCount === 0;
  const totalPathCount = pathFieldMeta.length;
  $: configuredPathCount = pathFieldMeta.reduce((count, entry) => {
    const field = entry.field;
    const hasValue = config[field].trim().length > 0;
    const hasNoErrors = pathFieldIssues[field].errors.length === 0;
    return count + (hasValue && hasNoErrors ? 1 : 0);
  }, 0);

  onMount(async () => {
    try {
      config = await invoke<AppConfig>("get_app_config");
      savedConfig = { ...config };
      pathValidation = await invoke<PathValidationResult>("preview_validate_config_paths", { newConfig: config });
      readyForLiveValidation = true;
    } catch (e) {
      console.error("Erreur chargement config:", e);
    }
  });

  onDestroy(() => {
    if (validationDebounceTimer !== null) {
      clearTimeout(validationDebounceTimer);
      validationDebounceTimer = null;
    }
  });

  async function refreshLiveValidation() {
    try {
      pathValidation = await invoke<PathValidationResult>("preview_validate_config_paths", { newConfig: config });
    } catch (e) {
      console.error("Erreur validation chemins:", e);
    }
  }

  function scheduleLiveValidation() {
    if (!readyForLiveValidation) return;
    if (validationDebounceTimer !== null) {
      clearTimeout(validationDebounceTimer);
    }
    validationDebounceTimer = setTimeout(() => {
      validationDebounceTimer = null;
      refreshLiveValidation();
    }, 180);
  }

  $: pathSignature = pathFieldMeta.map((entry) => config[entry.field]).join("|");
  $: if (readyForLiveValidation && pathSignature !== undefined) {
    scheduleLiveValidation();
  }

  async function pickPath(field: PathField) {
    const fieldMeta = pathFieldMeta.find((entry) => entry.field === field);
    const title = fieldMeta ? `Sélectionner: ${fieldMeta.label}` : "Sélectionner";
    const isFile = filePickerFields.includes(field);
    let result: string | string[] | null;
    if (isFile) {
      let filters;
      if (field === "presets_path" || field === "revision_tags_path") {
        filters = [{ name: "JSON", extensions: ["json"] }];
      } else {
        filters = [{ name: "Log", extensions: ["log"] }];
      }
      result = await open({ directory: false, multiple: false, title, filters });
    } else {
      result = await open({ directory: true, title });
    }
    if (result) {
      config[field] = result as string;
    }
  }

  async function saveConfig() {
    saving = true;
    status = "";
    statusType = "";
    try {
      await invoke("update_app_config", { newConfig: config });
      // Créer automatiquement presets et audit.log si absents
      await invoke("ensure_default_files").catch(() => {});
      pathValidation = await invoke<PathValidationResult>("preview_validate_config_paths", { newConfig: config });
      savedConfig = { ...config };
      status = "Configuration sauvegardée.";
      statusType = "success";
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    } finally {
      saving = false;
    }
  }
</script>

<main class="settings-page">
  <section class="settings-shell">
    <header class="settings-hero">
      <div class="settings-hero-content">
        <p class="settings-overline">Administration</p>
        <h1>Paramètres</h1>
        <p class="settings-hero-subtitle">Configuration des chemins, de la validation et de la sécurité de l'application.</p>
      </div>

      <div class="settings-hero-metrics">
        <div class="settings-health-chip" class:ok={validationIsHealthy}>
          {#if validationIsHealthy}Configuration valide{:else}Vérification requise{/if}
        </div>
        <div class="settings-kpi-tile">
          <span>Chemins configurés</span>
          <strong>{configuredPathCount} / {totalPathCount}</strong>
        </div>
      </div>
    </header>

    <div class="settings-grid">
      <section class="settings-card settings-card-main">
        <div class="card-head card-head-main">
          <div>
            <h2>Chemins de stockage</h2>
            <p>Définissez les emplacements utilisés par les espaces Dépôt, Validation et Archive.</p>
          </div>
        </div>

        <div class="settings-fields">
          {#each pathFieldMeta as fieldMeta}
            <article
              class="path-item"
              class:path-item-error={pathFieldIssues[fieldMeta.field].errors.length > 0}
              class:path-item-warning={pathFieldIssues[fieldMeta.field].errors.length === 0 &&
                pathFieldIssues[fieldMeta.field].warnings.length > 0}
            >
              <div class="path-item-head">
                <label class="meta-label">{fieldMeta.label}</label>
                <p class="field-help">{fieldMeta.hint}</p>
              </div>
              <div class="path-row">
                <input
                  class="meta-input"
                  class:input-error={pathFieldIssues[fieldMeta.field].errors.length > 0}
                  class:input-warning={pathFieldIssues[fieldMeta.field].errors.length === 0 &&
                    pathFieldIssues[fieldMeta.field].warnings.length > 0}
                  bind:value={config[fieldMeta.field]}
                />
                <button class="btn btn-secondary path-browse-btn" on:click={() => pickPath(fieldMeta.field)}
                  >Parcourir</button
                >
              </div>
              {#if savedConfig && savedConfig[fieldMeta.field] && savedConfig[fieldMeta.field] !== config[fieldMeta.field]}
                <div class="previous-path">
                  <span>Chemin précédent :</span>
                  <button
                    class="restore-btn"
                    title="Restaurer ce chemin"
                    on:click={() => { if (savedConfig) config[fieldMeta.field] = savedConfig[fieldMeta.field]; }}
                  >{savedConfig[fieldMeta.field]}</button>
                </div>
              {/if}
              {#if pathFieldIssues[fieldMeta.field].errors.length > 0}
                <div class="field-messages">
                  {#each pathFieldIssues[fieldMeta.field].errors as err}
                    <div class="field-message field-message-error">{err}</div>
                  {/each}
                </div>
              {:else if pathFieldIssues[fieldMeta.field].warnings.length > 0}
                <div class="field-messages">
                  {#each pathFieldIssues[fieldMeta.field].warnings as warn}
                    <div class="field-message field-message-warning">{warn}</div>
                  {/each}
                </div>
              {/if}
            </article>
          {/each}
        </div>
      </section>

      <aside class="settings-side">
        <section class="settings-card settings-card-security">
          <div class="card-head">
            <h2>Sécurité</h2>
            <p>Accès protégé aux actions administrateur.</p>
          </div>
          <div class="settings-field">
            <label class="meta-label">Mot de passe administrateur</label>
            <input class="meta-input" type="password" autocomplete="new-password" bind:value={config.admin_password} />
            <p class="inline-help">Conservez un mot de passe robuste pour sécuriser l'accès aux opérations sensibles.</p>
          </div>
        </section>

        <section class="settings-card settings-card-validation">
          <div class="card-head">
            <h2>État des chemins</h2>
            <p>Contrôle de cohérence de la configuration actuelle.</p>
          </div>

          {#if pathValidation}
            <div class="validation-kpis">
              <div class="validation-kpi">
                <span class="kpi-label">Erreurs</span>
                <span class="kpi-value">{validationErrorCount}</span>
              </div>
              <div class="validation-kpi">
                <span class="kpi-label">Avertissements</span>
                <span class="kpi-value">{validationWarningCount}</span>
              </div>
            </div>

            {#if globalValidationErrors.length > 0}
              <div class="validation-results">
                <p class="validation-title">Erreurs détectées</p>
                {#each globalValidationErrors as err}
                  <div class="status-message error">{err}</div>
                {/each}
              </div>
            {/if}

            {#if globalValidationWarnings.length > 0}
              <div class="validation-results">
                <p class="validation-title">Avertissements</p>
                {#each globalValidationWarnings as warn}
                  <div class="status-message warning">{warn}</div>
                {/each}
              </div>
            {/if}

            {#if validationErrorCount === 0 && validationWarningCount === 0}
              <div class="status-message success">Aucune anomalie détectée sur les chemins configurés.</div>
            {/if}
          {:else}
            <p class="validation-loading">Analyse des chemins en cours...</p>
          {/if}
        </section>

        <div class="settings-side-actions">
          <button class="btn btn-primary settings-save-btn" on:click={saveConfig} disabled={saving}>
            {saving ? "Sauvegarde..." : "Sauvegarder"}
          </button>
        </div>
      </aside>
    </div>

    <footer class="settings-actions-bar">
      <div class="settings-status-slot">
        {#if status}
          <div class="status-message status-inline {statusType}">{status}</div>
        {:else}
          <p class="status-placeholder">Aucune modification enregistrée pour le moment.</p>
        {/if}
      </div>
    </footer>
  </section>
</main>

<style>
  .settings-page {
    padding: var(--spacing-2xl);
    max-width: 1280px;
    margin: 0 auto;
  }

  .settings-shell {
    display: grid;
    gap: var(--spacing-lg);
  }

  .settings-hero {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-xl);
    padding: var(--spacing-xl);
    border: 1px solid var(--color-neutral-400);
    border-radius: var(--border-radius-lg);
    background: linear-gradient(180deg, #ffffff 0%, #fbfdff 100%);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.05);
  }

  .settings-overline {
    margin: 0 0 6px;
    font-size: 0.74rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-neutral-600);
  }

  .settings-hero-content h1 {
    margin: 0;
    color: var(--color-neutral-900);
    font-size: var(--font-size-xl);
    line-height: 1.2;
  }

  .settings-hero-subtitle {
    margin: 8px 0 0;
    max-width: 58ch;
    color: var(--color-neutral-700);
    font-size: var(--font-size-sm);
    line-height: 1.45;
  }

  .settings-hero-metrics {
    display: grid;
    justify-items: end;
    gap: var(--spacing-sm);
  }

  .settings-health-chip {
    padding: 6px 10px;
    border-radius: 999px;
    border: 1px solid rgba(148, 163, 184, 0.45);
    background: rgba(248, 250, 252, 0.95);
    color: var(--color-neutral-700);
    font-size: 0.74rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    white-space: nowrap;
  }

  .settings-health-chip.ok {
    border-color: rgba(34, 197, 94, 0.3);
    background: rgba(240, 253, 244, 0.95);
    color: #166534;
  }

  .settings-kpi-tile {
    min-width: 168px;
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-neutral-300);
    background: #fff;
    padding: 8px 12px;
    display: grid;
    gap: 2px;
    text-align: right;
  }

  .settings-kpi-tile span {
    color: var(--color-neutral-600);
    font-size: 0.72rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-weight: 700;
  }

  .settings-kpi-tile strong {
    color: var(--color-neutral-900);
    font-size: 1.02rem;
    font-weight: 700;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.55fr) minmax(320px, 0.95fr);
    gap: var(--spacing-lg);
    align-items: start;
  }

  .settings-side {
    display: grid;
    gap: var(--spacing-lg);
  }

  .settings-card {
    background: var(--color-neutral-100);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-xl);
    border: 1px solid var(--color-neutral-400);
    box-shadow: var(--shadow-sm);
  }

  .card-head {
    margin-bottom: var(--spacing-lg);
  }

  .card-head-main {
    display: flex;
    justify-content: space-between;
    gap: var(--spacing-lg);
    align-items: flex-start;
  }

  .card-head h2 {
    margin: 0;
    color: var(--color-neutral-900);
    font-size: var(--font-size-lg);
    line-height: 1.2;
  }

  .card-head p {
    margin: 6px 0 0;
    color: var(--color-neutral-700);
    font-size: 0.83rem;
    line-height: 1.4;
  }


  .settings-fields {
    display: grid;
    gap: var(--spacing-sm);
  }

  .path-item {
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    padding: 10px;
    background: #fff;
    display: grid;
    gap: 8px;
  }

  .path-item-error {
    border-color: var(--color-error-border);
    background: #fffafb;
  }

  .path-item-warning {
    border-color: #f2c66d;
    background: #fffdf7;
  }

  .path-item-head {
    display: grid;
    gap: 2px;
  }

  .field-help {
    margin: 0;
    color: var(--color-neutral-600);
    font-size: 0.74rem;
    line-height: 1.35;
  }

  .path-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .path-browse-btn {
    min-height: 36px;
    white-space: nowrap;
  }

  .input-error {
    border-color: var(--color-error-border);
    box-shadow: 0 0 0 2px rgba(252, 129, 129, 0.14);
  }

  .input-warning {
    border-color: #f2c66d;
    box-shadow: 0 0 0 2px rgba(242, 198, 109, 0.15);
  }

  .previous-path {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-size: 0.72rem;
    color: var(--color-neutral-600);
  }

  .restore-btn {
    background: none;
    border: none;
    padding: 0;
    font-size: 0.72rem;
    color: var(--color-primary);
    cursor: pointer;
    font-family: ui-monospace, monospace;
    text-decoration: underline;
    text-align: left;
    word-break: break-all;
  }

  .restore-btn:hover {
    color: var(--color-primary-hover);
  }

  .field-messages {
    display: grid;
    gap: 6px;
  }

  .field-message {
    border-radius: var(--border-radius-sm);
    border: 1px solid transparent;
    padding: 6px 8px;
    font-size: 0.76rem;
    line-height: 1.4;
  }

  .field-message-error {
    border-color: var(--color-error-border);
    background: var(--color-error-bg);
    color: var(--color-error);
  }

  .field-message-warning {
    border-color: #f2c66d;
    background: #fff7e6;
    color: #8a5a0a;
  }

  .settings-field {
    display: grid;
    gap: 8px;
  }

  .inline-help {
    margin: 0;
    color: var(--color-neutral-600);
    font-size: 0.76rem;
    line-height: 1.4;
  }

  .validation-kpis {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .validation-kpi {
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    background: #fff;
    padding: 8px 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .kpi-label {
    color: var(--color-neutral-600);
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .kpi-value {
    color: var(--color-neutral-900);
    font-size: 0.96rem;
    font-weight: 800;
  }

  .validation-results {
    display: grid;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }

  .validation-title {
    margin: 0;
    color: var(--color-neutral-700);
    font-size: 0.74rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .validation-loading {
    margin: 0;
    color: var(--color-neutral-700);
    font-size: var(--font-size-sm);
  }

  .settings-side-actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
  }

  .settings-actions-bar {
    display: flex;
    justify-content: flex-start;
    align-items: center;
    gap: var(--spacing-md);
    border-top: 1px solid var(--color-neutral-300);
    padding-top: var(--spacing-md);
  }

  .settings-status-slot {
    min-height: 34px;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .status-inline {
    margin: 0;
    width: 100%;
  }

  .status-placeholder {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-600);
  }

  .settings-save-btn {
    min-width: 148px;
  }

  @media (max-width: 1080px) {
    .settings-grid {
      grid-template-columns: 1fr;
    }

    .settings-side {
      grid-template-columns: 1fr 1fr;
      gap: var(--spacing-md);
    }
  }

  @media (max-width: 840px) {
    .settings-page {
      padding: var(--spacing-lg);
    }

    .settings-hero {
      flex-direction: column;
      align-items: stretch;
      gap: var(--spacing-md);
      padding: var(--spacing-lg);
    }

    .settings-hero-metrics {
      justify-items: start;
    }

    .settings-kpi-tile {
      text-align: left;
    }

    .settings-card {
      padding: var(--spacing-lg);
    }

    .card-head-main {
      flex-direction: column;
      gap: var(--spacing-sm);
    }

    .settings-side {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .path-row {
      grid-template-columns: 1fr;
    }

    .settings-actions-bar {
      flex-direction: column;
      align-items: stretch;
    }

    .settings-save-btn {
      width: 100%;
    }

    .settings-side-actions {
      justify-content: stretch;
    }
  }
</style>
