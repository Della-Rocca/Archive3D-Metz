<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { OperationMeta, Presets } from "$lib/types/deposit";
  import { createEventDispatcher } from "svelte";
  import ComboInput from "$lib/components/ComboInput.svelte";
  import { getSafeSegmentError, isSafeSegment } from "$lib/utils/path";

  const dispatch = createEventDispatcher();

  // --- Types ---

  type StringCategory =
    | "structure_types"
    | "operation_types"
    | "software_types"
    | "sites"
    | "responsables"
    | "model_authors"
    | "depositors";

  interface CategoryConfig {
    key: StringCategory;
    label: string;
    placeholder: string;
  }

  // --- Config des catégories ---
  const categories: CategoryConfig[] = [
    {
      key: "structure_types",
      label: "Types de structure",
      placeholder: "ex: sépulture, mur, fosse...",
    },
    {
      key: "operation_types",
      label: "Types d'opération",
      placeholder: "ex: diagnostic, fouille...",
    },
    {
      key: "software_types",
      label: "Logiciels",
      placeholder: "ex: Metashape, RealityCapture...",
    },
    {
      key: "sites",
      label: "Sites",
      placeholder: "ex: Metz-centre, Metz-nord...",
    },
    {
      key: "responsables",
      label: "Responsables",
      placeholder: "ex: Dupont Jean...",
    },
    {
      key: "model_authors",
      label: "Auteurs de modèles",
      placeholder: "ex: Martin Pierre...",
    },
    { key: "depositors", label: "Déposants", placeholder: "ex: Votre nom..." },
  ];

  // Types d'opération fixes
  const operationTypeChoices = ["diagnostic", "fouille"];

  // --- State ---
  let presets: Presets = {
    operations: [],
    structure_types: [],
    operation_types: [],
    software_types: [],
    sites: [],
    responsables: [],
    model_authors: [],
    depositors: [],
  };

  let activeTab: "categories" | "operations" = "categories";
  let activeCategory: StringCategory = "structure_types";
  let newValue = "";
  let saving = false;
  let loading = true;
  let status = "";
  let statusType: "success" | "error" | "" = "";

  // Opérations : champs pour ajouter
  let newOp: OperationMeta = {
    code: "",
    site: "",
    op_type: "",
    responsable: "",
  };

  $: newValueError =
    activeCategory === "sites" ? getSafeSegmentError(newValue) : "";
  $: newOpCodeError = getSafeSegmentError(newOp.code);
  $: newOpSiteError = getSafeSegmentError(newOp.site);

  // --- Chargement ---
  async function loadPresets() {
    loading = true;
    try {
      presets = await invoke<Presets>("get_metadata_presets");
    } catch (e) {
      status = `Erreur chargement : ${e}`;
      statusType = "error";
    } finally {
      loading = false;
    }
  }

  loadPresets();

  // --- Sauvegarde ---
  async function savePresets() {
    saving = true;
    status = "";
    statusType = "";
    try {
      await invoke("update_metadata_presets", { presets });
      status = "Métadonnées sauvegardées avec succès !";
      statusType = "success";
      dispatch("saved");
    } catch (e) {
      status = `Erreur sauvegarde : ${e}`;
      statusType = "error";
    } finally {
      saving = false;
    }
  }

  // --- Ajout / Suppression de valeurs string ---
  function addStringValue() {
    const trimmed = newValue.trim();
    if (!trimmed) return;
    if (activeCategory === "sites" && !isSafeSegment(trimmed)) {
      status = newValueError;
      statusType = "error";
      return;
    }
    if (presets[activeCategory].includes(trimmed)) {
      status = `"${trimmed}" existe déjà dans cette catégorie.`;
      statusType = "error";
      return;
    }
    presets[activeCategory] = [...presets[activeCategory], trimmed];
    newValue = "";
    status = "";
    statusType = "";
  }

  function removeStringValue(index: number) {
    presets[activeCategory] = presets[activeCategory].filter(
      (_, i) => i !== index,
    );
  }

  // --- Ajout / Suppression d'opérations ---
  function addOperation() {
    if (
      !newOp.code.trim() ||
      !newOp.site.trim() ||
      !newOp.op_type.trim() ||
      !newOp.responsable.trim()
    ) {
      status = "Tous les champs sont obligatoires pour une opération.";
      statusType = "error";
      return;
    }
    if (!/^\d+$/.test(newOp.code.trim())) {
      status = "Le code d'opération doit contenir uniquement des chiffres.";
      statusType = "error";
      return;
    }
    if (!isSafeSegment(newOp.code.trim()) || !isSafeSegment(newOp.site.trim())) {
      status = newOpCodeError || newOpSiteError;
      statusType = "error";
      return;
    }
    if (presets.operations.some((op) => op.code === newOp.code.trim())) {
      status = `L'opération "${newOp.code.trim()}" existe déjà.`;
      statusType = "error";
      return;
    }

    const site = newOp.site.trim();
    const responsable = newOp.responsable.trim();
    const opType = newOp.op_type.trim();

    // Ajouter l'opération
    presets.operations = [
      ...presets.operations,
      {
        code: newOp.code.trim(),
        site: site,
        op_type: opType,
        responsable: responsable,
      },
    ];

    // Enrichir automatiquement les presets associés
    if (site && !presets.sites.includes(site)) {
      presets.sites = [...presets.sites, site];
    }
    if (responsable && !presets.responsables.includes(responsable)) {
      presets.responsables = [...presets.responsables, responsable];
    }
    if (opType && !presets.operation_types.includes(opType)) {
      presets.operation_types = [...presets.operation_types, opType];
    }

    newOp = { code: "", site: "", op_type: "", responsable: "" };
    status = "";
    statusType = "";
  }

  function removeOperation(index: number) {
    presets.operations = presets.operations.filter((_, i) => i !== index);
  }

  // --- Fermeture ---
  function handleClose() {
    dispatch("close");
  }

  function handleAddKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      addStringValue();
    }
  }
</script>

<div class="editor-page">
  <!-- Header pleine largeur -->
  <div class="editor-header">
    <div class="editor-header-left">
      <button class="back-btn" on:click={handleClose} title="Retour au dépôt">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
        >
          <path
            d="M19 12H5M5 12L12 19M5 12L12 5"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <div>
        <h1>Édition des métadonnées</h1>
        <p class="editor-subtitle">
          Gérez les listes de valeurs prédéfinies utilisées dans les
          formulaires.
        </p>
      </div>
    </div>
    <div class="editor-header-actions">
      <button class="btn btn-secondary" on:click={handleClose}>
        Annuler
      </button>
      <button class="btn btn-primary" on:click={savePresets} disabled={saving}>
        {saving ? "Sauvegarde..." : "Sauvegarder"}
      </button>
    </div>
  </div>

  {#if status}
    <div class="status-message {statusType}">{status}</div>
  {/if}

  {#if loading}
    <div class="loading-container">
      <p class="loading-text">Chargement des métadonnées...</p>
    </div>
  {:else}
    <!-- Onglets principaux -->
    <div class="main-tabs">
      <button
        class="main-tab"
        class:active={activeTab === "categories"}
        on:click={() => (activeTab = "categories")}
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
        >
          <path
            d="M4 6H20M4 12H20M4 18H12"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
        Catégories
      </button>
      <button
        class="main-tab"
        class:active={activeTab === "operations"}
        on:click={() => (activeTab = "operations")}
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
        >
          <path
            d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        Opérations
        <span class="tab-count">{presets.operations.length}</span>
      </button>
    </div>

    <!-- ========== CONTENU CATÉGORIES ========== -->
    {#if activeTab === "categories"}
      <div class="editor-layout">
        <!-- Barre latérale catégories -->
        <aside class="category-sidebar">
          <h3 class="sidebar-title">Catégories</h3>
          {#each categories as cat}
            <button
              class="sidebar-item"
              class:active={activeCategory === cat.key}
              on:click={() => {
                activeCategory = cat.key;
                newValue = "";
                status = "";
                statusType = "";
              }}
            >
              <span class="sidebar-label">{cat.label}</span>
              <span class="sidebar-count">{presets[cat.key].length}</span>
            </button>
          {/each}
        </aside>

        <!-- Contenu principal -->
        <div class="editor-content">
          <div class="content-header">
            <h2>
              {categories.find((c) => c.key === activeCategory)?.label ?? ""}
            </h2>
            <span class="content-count"
              >{presets[activeCategory].length} élément{presets[activeCategory]
                .length !== 1
                ? "s"
                : ""}</span
            >
          </div>

          <!-- Champ d'ajout -->
          <div class="add-row">
            <input
              class="meta-input add-input"
              class:input-error={!!newValueError}
              bind:value={newValue}
              placeholder={categories.find((c) => c.key === activeCategory)
                ?.placeholder ?? "Nouvelle valeur..."}
              on:keydown={handleAddKeydown}
            />
            <button
              class="btn btn-primary add-btn"
              on:click={addStringValue}
              disabled={!newValue.trim() || !!newValueError}
            >
              + Ajouter
            </button>
          </div>
          {#if newValueError}
            <p class="field-error">{newValueError}</p>
          {/if}

          <!-- Liste des valeurs -->
          <div class="values-list">
            {#if presets[activeCategory].length === 0}
              <div class="empty-state">
                <p class="empty-text">Aucune valeur dans cette catégorie.</p>
                <p class="empty-hint">
                  Utilisez le champ ci-dessus pour en ajouter.
                </p>
              </div>
            {:else}
              {#each presets[activeCategory] as value, i}
                <div class="value-item">
                  <span class="value-text">{value}</span>
                  <button
                    class="remove-btn"
                    on:click={() => removeStringValue(i)}
                    title="Supprimer « {value} »"
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      xmlns="http://www.w3.org/2000/svg"
                      width="16"
                      height="16"
                    >
                      <path
                        d="M18 6L6 18M6 6L18 18"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                      />
                    </svg>
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- ========== CONTENU OPÉRATIONS ========== -->
    {#if activeTab === "operations"}
      <div class="operations-content">
        <!-- Formulaire d'ajout d'opération -->
        <div class="op-add-form">
          <h3>Ajouter une nouvelle opération</h3>
          <div class="op-fields">
            <div class="op-field">
              <label class="meta-label">Code *</label>
              <input
                class="meta-input"
                class:input-error={!!newOpCodeError}
                bind:value={newOp.code}
                placeholder="ex: 202501"
                inputmode="numeric"
                pattern="[0-9]*"
                on:input={(e) => {
                  newOp.code = e.currentTarget.value.replace(/\D/g, "");
                }}
              />
              {#if newOpCodeError}
                <p class="field-error">{newOpCodeError}</p>
              {/if}
            </div>
            <div class="op-field">
              <label class="meta-label">Site *</label>
              <div class:combo-error={!!newOpSiteError}>
                <ComboInput
                  bind:value={newOp.site}
                  options={presets.sites}
                  placeholder="Saisir ou sélectionner un site..."
                />
              </div>
              {#if newOpSiteError}
                <p class="field-error">{newOpSiteError}</p>
              {/if}
            </div>
            <div class="op-field">
              <label class="meta-label">Type d'opération *</label>
              <div class="meta-select-wrapper">
                <select class="meta-select" bind:value={newOp.op_type}>
                  <option value="">-- Sélectionner un type --</option>
                  {#each operationTypeChoices as t}
                    <option value={t}
                      >{t.charAt(0).toUpperCase() + t.slice(1)}</option
                    >
                  {/each}
                </select>
              </div>
            </div>
            <div class="op-field">
              <label class="meta-label">Responsable *</label>
              <ComboInput
                bind:value={newOp.responsable}
                options={presets.responsables}
                placeholder="Saisir ou sélectionner un responsable..."
              />
            </div>
          </div>
          <button
            class="btn btn-primary"
            on:click={addOperation}
            disabled={!newOp.code.trim() ||
              !!newOpCodeError ||
              !!newOpSiteError ||
              !newOp.site.trim() ||
              !newOp.op_type.trim() ||
              !newOp.responsable.trim()}
          >
            + Ajouter l'opération
          </button>
        </div>

        <!-- Liste des opérations -->
        <div class="op-list-header">
          <h3>Opérations enregistrées</h3>
          <span class="content-count"
            >{presets.operations.length} opération{presets.operations.length !==
            1
              ? "s"
              : ""}</span
          >
        </div>

        <div class="op-list">
          {#if presets.operations.length === 0}
            <div class="empty-state">
              <p class="empty-text">Aucune opération enregistrée.</p>
              <p class="empty-hint">
                Utilisez le formulaire ci-dessus pour en ajouter.
              </p>
            </div>
          {:else}
            {#each presets.operations as op, i}
              <div class="op-item">
                <div class="op-info">
                  <div class="op-main">
                    <span class="op-code">{op.code}</span>
                    <span class="op-separator">—</span>
                    <span class="op-site">{op.site}</span>
                  </div>
                  <div class="op-details">
                    {#if op.op_type}
                      <span class="op-tag op-tag-type">{op.op_type}</span>
                    {/if}
                    {#if op.responsable}
                      <span class="op-tag op-tag-resp">{op.responsable}</span>
                    {/if}
                  </div>
                </div>
                <button
                  class="remove-btn"
                  on:click={() => removeOperation(i)}
                  title="Supprimer l'opération {op.code}"
                >
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                  >
                    <path
                      d="M18 6L6 18M6 6L18 18"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                    />
                  </svg>
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  /* ===== PAGE CONTENEUR ===== */
  .editor-page {
    animation: fadeIn 0.25s ease;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ===== HEADER ===== */
  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-lg) var(--spacing-xl);
    background: linear-gradient(135deg, var(--color-primary) 0%, #2a4f7a 100%);
    border-radius: var(--border-radius-lg);
    color: var(--color-neutral-100);
    margin-bottom: var(--spacing-xl);
  }

  .editor-header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
  }

  .back-btn {
    background: rgba(255, 255, 255, 0.15);
    border: none;
    color: var(--color-neutral-100);
    width: 40px;
    height: 40px;
    border-radius: var(--border-radius-md);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    flex-shrink: 0;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .editor-header h1 {
    margin: 0;
    font-size: var(--font-size-xl);
    font-weight: 700;
  }

  .editor-subtitle {
    margin: 2px 0 0;
    font-size: var(--font-size-sm);
    opacity: 0.8;
  }

  .editor-header-actions {
    display: flex;
    gap: var(--spacing-sm);
    flex-shrink: 0;
  }

  /* ===== ONGLETS PRINCIPAUX ===== */
  .main-tabs {
    display: flex;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-xl);
    border-bottom: 2px solid var(--color-neutral-300);
    padding-bottom: 0;
  }

  .main-tab {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-md) var(--spacing-xl);
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: var(--font-size-base);
    font-weight: 600;
    font-family: var(--font-family);
    color: var(--color-neutral-600);
    transition: all 0.2s;
    border-bottom: 3px solid transparent;
    margin-bottom: -2px;
  }

  .main-tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .main-tab:hover:not(.active) {
    color: var(--color-neutral-900);
  }

  .tab-count {
    background: var(--color-neutral-300);
    padding: 2px 8px;
    border-radius: 999px;
    font-size: var(--font-size-xs);
    font-weight: 700;
  }

  .main-tab.active .tab-count {
    background: rgba(30, 58, 95, 0.1);
    color: var(--color-primary);
  }

  /* ===== LAYOUT CATÉGORIES (sidebar + contenu) ===== */
  .editor-layout {
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: var(--spacing-xl);
    min-height: 500px;
  }

  /* ===== SIDEBAR ===== */
  .category-sidebar {
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid var(--color-neutral-300);
    height: fit-content;
    position: sticky;
    top: var(--spacing-xl);
  }

  .sidebar-title {
    margin: 0 0 var(--spacing-md);
    font-size: var(--font-size-xs);
    font-weight: 700;
    color: var(--color-neutral-600);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    background: transparent;
    border-radius: var(--border-radius-md);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-family: var(--font-family);
    color: var(--color-neutral-700);
    transition: all 0.15s;
    text-align: left;
  }

  .sidebar-item:hover {
    background: var(--color-neutral-300);
    color: var(--color-neutral-900);
  }

  .sidebar-item.active {
    background: var(--color-primary);
    color: var(--color-neutral-100);
    font-weight: 600;
  }

  .sidebar-label {
    flex: 1;
  }

  .sidebar-count {
    background: rgba(0, 0, 0, 0.08);
    padding: 1px 7px;
    border-radius: 999px;
    font-size: 0.65rem;
    font-weight: 700;
    flex-shrink: 0;
  }

  .sidebar-item.active .sidebar-count {
    background: rgba(255, 255, 255, 0.25);
  }

  /* ===== CONTENU PRINCIPAL ===== */
  .editor-content {
    min-width: 0;
  }

  .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-lg);
  }

  .content-header h2 {
    margin: 0;
    font-size: var(--font-size-lg);
    color: var(--color-neutral-900);
  }

  .content-count {
    font-size: var(--font-size-sm);
    color: var(--color-neutral-600);
    background: var(--color-neutral-200);
    padding: 2px 10px;
    border-radius: 999px;
    font-weight: 600;
  }

  /* ===== CHAMP D'AJOUT ===== */
  .add-row {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-xl);
  }

  .field-error {
    margin: calc(var(--spacing-sm) * -0.5) 0 var(--spacing-md);
    font-size: var(--font-size-xs);
    color: var(--color-error);
    font-weight: 500;
  }

  .add-input {
    flex: 1;
  }

  .add-btn {
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* ===== LISTE DES VALEURS ===== */
  .values-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .value-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--color-neutral-100);
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-neutral-300);
    transition: all 0.15s;
  }

  .value-item:hover {
    border-color: var(--color-primary);
    box-shadow: 0 2px 8px rgba(30, 58, 95, 0.06);
  }

  .value-text {
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
    font-weight: 500;
  }

  .remove-btn {
    background: transparent;
    border: none;
    color: var(--color-neutral-500);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--border-radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    opacity: 0;
  }

  .value-item:hover .remove-btn,
  .op-item:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    background: var(--color-error-bg);
    color: var(--color-error);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-2xl) var(--spacing-xl);
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-lg);
    border: 2px dashed var(--color-neutral-400);
  }

  .empty-text {
    color: var(--color-neutral-700);
    font-weight: 600;
    font-size: var(--font-size-base);
    margin: 0;
  }

  .empty-hint {
    color: var(--color-neutral-600);
    font-size: var(--font-size-sm);
    margin: var(--spacing-xs) 0 0;
  }

  /* ===== CONTENU OPÉRATIONS ===== */
  .operations-content {
    max-width: 900px;
  }

  .op-add-form {
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-xl);
    margin-bottom: var(--spacing-2xl);
    border: 1px solid var(--color-neutral-300);
  }

  .op-add-form h3 {
    margin: 0 0 var(--spacing-lg);
    font-size: var(--font-size-base);
    color: var(--color-primary);
    font-weight: 700;
  }

  .op-fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .op-field label {
    margin-bottom: 4px;
  }

  .meta-select {
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23666' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    padding-right: 32px;
    cursor: pointer;
  }

  .op-list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-md);
  }

  .op-list-header h3 {
    margin: 0;
    font-size: var(--font-size-base);
    color: var(--color-neutral-900);
  }

  .op-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .op-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--color-neutral-100);
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-neutral-300);
    transition: all 0.15s;
  }

  .op-item:hover {
    border-color: var(--color-primary);
    box-shadow: 0 2px 8px rgba(30, 58, 95, 0.06);
  }

  .op-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .op-main {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .op-code {
    font-weight: 700;
    font-size: var(--font-size-sm);
    color: var(--color-primary);
    background: rgba(30, 58, 95, 0.08);
    padding: 3px 10px;
    border-radius: var(--border-radius-sm);
  }

  .op-separator {
    color: var(--color-neutral-400);
    font-size: var(--font-size-sm);
  }

  .op-site {
    font-weight: 600;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
  }

  .op-details {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .op-tag {
    font-size: var(--font-size-xs);
    padding: 2px 8px;
    border-radius: var(--border-radius-sm);
  }

  .op-tag-type {
    color: var(--color-primary);
    background: rgba(30, 58, 95, 0.06);
    border: 1px solid rgba(30, 58, 95, 0.12);
  }

  .op-tag-resp {
    color: var(--color-neutral-700);
    background: var(--color-neutral-200);
    border: 1px solid var(--color-neutral-300);
    font-style: italic;
  }

  /* ===== LOADING ===== */
  .loading-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
  }

  .loading-text {
    text-align: center;
    color: var(--color-neutral-600);
    font-style: italic;
    font-size: var(--font-size-base);
  }

  /* ===== STATUS ===== */
  .status-message {
    margin-bottom: var(--spacing-lg);
  }
</style>
