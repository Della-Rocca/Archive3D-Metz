<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick } from "svelte";
  import ComboInput from "$lib/components/ComboInput.svelte";
  import DepositCard from "$lib/components/DepositCard.svelte";
  import { getSafeSegmentError, isSafeSegment } from "$lib/utils/path";

  import type { StructureListItem } from "$lib/types/fs";
  import type {
    DepositMetadata,
    StructureDetails,
    Presets,
  } from "$lib/types/deposit";

  interface AuditEntry {
    timestamp: string;
    action: string;
    user: string;
    structure_path: string;
    metadata: unknown | null;
    success: boolean;
    error: string | null;
  }

  // --- State ---
  let items: StructureListItem[] = [];
  let selectedItem: StructureListItem | null = null;
  let metadata: DepositMetadata | null = null;
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

  let loading = false;
  let processing = false;
  let status = "";
  let statusType: "success" | "error" | "warning" | "" = "";

  let filterType = "";
  let filterOperation = "";
  let filterRevision: "all" | "tagged" | "untagged" = "all";
  let searchQuery = "";
  let listDetails: Record<string, StructureDetails | null> = {};

  let showPreviewModal = false;
  let showArchiveConfirm = false;

  let isEditing = false;
  let editMetadata: DepositMetadata | null = null;
  let selectedSoftware: string[] = [];
  let activeTab: "pending" | "history" = "pending";
  let historyEntries: AuditEntry[] = [];
  let loadingHistory = false;
  let resettingHistory = false;
  let revisionTags: Record<string, { tagged: boolean; note: string; updated_at: string }> = {};
  let showCompactDetails = false;
  let previewBodyEl: HTMLDivElement | null = null;

  $: if (editMetadata) {
    editMetadata.structure.software = selectedSoftware.join(", ");
  }

  $: editFieldErrors = editMetadata
    ? {
        "operation.code": !editMetadata.operation.code.trim()
          ? "Champ obligatoire"
          : getSafeSegmentError(editMetadata.operation.code),
        "operation.site": !editMetadata.operation.site.trim()
          ? "Champ obligatoire"
          : getSafeSegmentError(editMetadata.operation.site),
        "structure.id": !editMetadata.structure.id.trim()
          ? "Champ obligatoire"
          : getSafeSegmentError(editMetadata.structure.id),
      }
    : {
        "operation.code": "",
        "operation.site": "",
        "structure.id": "",
      };

  $: editMetadataValid = !!editMetadata &&
    !!editMetadata.operation.code.trim() &&
    !!editMetadata.operation.site.trim() &&
    !!editMetadata.structure.id.trim() &&
    isSafeSegment(editMetadata.operation.code) &&
    isSafeSegment(editMetadata.operation.site) &&
    isSafeSegment(editMetadata.structure.id);

  onMount(async () => {
    await loadItems();
    await loadPresets();
    await loadRevisionTags();
  });

  async function loadRevisionTags() {
    try {
      revisionTags = await invoke("get_revision_tags");
    } catch (e) {
      console.error("Erreur chargement tags révision:", e);
    }
  }

  function applySearch() {
    searchQuery = searchQuery.trim();
  }

  async function loadHistory() {
    loadingHistory = true;
    try {
      historyEntries = await invoke<AuditEntry[]>("get_recent_validations", {
        limit: 50,
      });
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    } finally {
      loadingHistory = false;
    }
  }

  async function resetHistory() {
    resettingHistory = true;
    status = "";
    statusType = "";
    try {
      await invoke("reset_validation_history");
      historyEntries = [];
      status = "Historique de validation réinitialisé.";
      statusType = "success";
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    } finally {
      resettingHistory = false;
    }
  }

  async function switchTab(tab: "pending" | "history") {
    activeTab = tab;
    if (tab === "history") {
      await loadHistory();
    }
  }

  async function loadItems() {
    loading = true;
    try {
      items = await invoke<StructureListItem[]>("list_validation_items");
      preloadListMetadata(items);
    } catch (e) {
      console.error("Erreur chargement structures:", e);
    } finally {
      loading = false;
    }
  }

  async function loadPresets() {
    try {
      presets = await invoke<Presets>("get_metadata_presets");
    } catch (e) {
      console.error("Erreur chargement presets:", e);
    }
  }

  async function preloadListMetadata(list: StructureListItem[]) {
    const entries = await Promise.all(
      list.map(async (item) => {
        try {
          const details = await invoke<StructureDetails>(
            "get_structure_details",
            { structurePath: item.path },
          );
          return [item.path, details] as const;
        } catch {
          return [item.path, null] as const;
        }
      }),
    );
    listDetails = { ...listDetails, ...Object.fromEntries(entries) };
  }

  $: operationOptions = Array.from(
    new Set(items.map((i) => i.operation_folder)),
  ).sort();

  $: filteredItems = items.filter((item) => {
    const opOk = !filterOperation || item.operation_folder === filterOperation;
    const meta = listDetails[item.path]?.metadata ?? null;
    const typeOk = !filterType || meta?.structure.st_type === filterType;
    const tagged = !!revisionTags[item.path]?.tagged;
    const revisionOk =
      filterRevision === "all" ||
      (filterRevision === "tagged" && tagged) ||
      (filterRevision === "untagged" && !tagged);
    const q = searchQuery.trim().toLowerCase();
    const searchOk =
      !q ||
      item.structure_folder.toLowerCase().includes(q) ||
      item.operation_folder.toLowerCase().includes(q) ||
      (meta?.operation.code || "").toLowerCase().includes(q) ||
      (meta?.operation.site || "").toLowerCase().includes(q) ||
      (meta?.structure.st_type || "").toLowerCase().includes(q);
    return opOk && typeOk && revisionOk && searchOk;
  });

  async function selectItem(item: StructureListItem) {
    if (isEditing) cancelEdit();
    selectedItem = item;
    metadata = null;
    status = "";
    statusType = "";
    isEditing = false;
    showPreviewModal = true;
    showArchiveConfirm = false;
    showCompactDetails = false;

    try {
      const details = await invoke<StructureDetails>("get_structure_details", {
        structurePath: item.path,
      });
      metadata = details.metadata;
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    }
  }

  function startEdit() {
    if (!metadata) return;
    editMetadata = JSON.parse(JSON.stringify(metadata));
    selectedSoftware = editMetadata?.structure.software
      ? editMetadata.structure.software.split(", ").filter((s) => s)
      : [];
    isEditing = true;
    status = "";
  }

  async function openEditModal() {
    startEdit();
    showPreviewModal = true;
    await tick();
    previewBodyEl?.scrollTo({ top: 0, behavior: "auto" });
  }

  function cancelEdit() {
    isEditing = false;
    editMetadata = null;
    status = "";
  }

  function toggleSoftware(sw: string) {
    if (selectedSoftware.includes(sw)) {
      selectedSoftware = selectedSoftware.filter((s) => s !== sw);
    } else {
      selectedSoftware = [...selectedSoftware, sw];
    }
  }

  async function saveMetadata() {
    if (!selectedItem || !editMetadata) return;
    if (!editMetadataValid) return;
    processing = true;
    try {
      await invoke("update_structure_metadata", {
        structurePath: selectedItem.path,
        updatedMetadata: editMetadata,
      });

      metadata = editMetadata;
      isEditing = false;
      status = "Métadonnées mises à jour.";
      statusType = "success";
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    } finally {
      processing = false;
    }
  }

  async function archiveNow() {
    if (!selectedItem) return;

    processing = true;
    status = "";
    statusType = "";

    try {
      await invoke("archive_structure_from_depot", {
        structurePathInDepot: selectedItem.path,
      });

      status = "Structure archivée avec succès.";
      statusType = "success";
      selectedItem = null;
      metadata = null;
      showPreviewModal = false;
      showArchiveConfirm = false;
      await loadItems();
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    } finally {
      processing = false;
    }
  }

  function openArchiveConfirm() {
    showArchiveConfirm = true;
  }

  async function toggleRevisionTag() {
    if (!selectedItem) return;
    processing = true;
    status = "";
    statusType = "";
    try {
      const tagged = !revisionTags[selectedItem.path]?.tagged;
      await invoke("set_revision_tag", {
        structurePathInDepot: selectedItem.path,
        tagged,
        note: tagged ? "À modifier plus tard" : "",
      });
      await loadRevisionTags();
      status = tagged
        ? "Structure marquée à modifier."
        : "Tag modification retiré.";
      statusType = "success";
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    } finally {
      processing = false;
    }
  }

  async function revealInExplorer(path: string) {
    if (!path) return;
    try {
      await invoke("reveal_in_explorer", { path });
    } catch (e) {
      status = `Erreur : ${e}`;
      statusType = "error";
    }
  }

  function structureName(path: string): string {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || parts[parts.length - 2] || path;
  }

  function formatTimestamp(ts: string): string {
    try {
      const d = new Date(ts);
      return `${d.toLocaleDateString("fr-FR")} ${d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })}`;
    } catch {
      return ts;
    }
  }

  function actionLabel(action: string): string {
    if (action === "overridearchive") return "Forcé";
    if (action === "archive") return "Archivé";
    if (action === "edit") return "Édition";
    return action;
  }

  function buildCompactPreview(path: string) {
    const details = listDetails[path] ?? null;
    const models = details?.models?.length || 0;
    const orthos = details?.orthos?.length || 0;
    const photos = details?.photos?.length || 0;
    const work = details?.work_files?.length || 0;
    const meta = details?.metadata ?? null;

    const errors: string[] = [];
    const warnings: string[] = [];

    let errorCount = 0;
    let warningCount = 0;

    // Erreurs bloquantes
    if (models === 0) {
      errorCount += 1;
      errors.push("Aucun modèle 3D détecté");
    }
    if (!meta?.operation?.code) {
      errorCount += 1;
      errors.push("Code opération manquant");
    }
    if (!meta?.structure?.id) {
      errorCount += 1;
      errors.push("Identifiant structure manquant");
    }

    // Warnings de complétude
    if (orthos === 0) {
      warningCount += 1;
      warnings.push("Aucune orthophotographie");
    }
    if (photos === 0) {
      warningCount += 1;
      warnings.push("Aucune photo");
    }
    if (work === 0) {
      warningCount += 1;
      warnings.push("Aucun fichier de travail");
    }
    if (!meta?.operation?.site) {
      warningCount += 1;
      warnings.push("Site non renseigné");
    }
    if (!meta?.structure?.st_type) {
      warningCount += 1;
      warnings.push("Type de structure manquant");
    }

    return { models, orthos, photos, work, errorCount, warningCount, errors, warnings };
  }
</script>

<main class="validation-page">
  <div class="page-header">
    <div class="header-left">
      <h1>Validation</h1>
      <p class="header-subtitle">
        Contrôlez la conformité des dépôts avant archivage.
      </p>
    </div>
    {#if activeTab === "pending"}
      <div class="pending-indicator">
        <span class="pending-count-wrap">
          <span class="pending-count">{items.length}</span>
        </span>
        <span class="pending-text">
          <span class="pending-label">Dépôts en attente</span>
          <span class="pending-subtitle">À valider maintenant</span>
        </span>
      </div>
    {/if}
  </div>

  <div class="tabs-bar">
    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === "pending"}
        on:click={() => switchTab("pending")}
      >
        <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
          ><path
            d="M12 6V12L16 14M22 12C22 17.5228 17.5228 22 12 22C6.47715 22 2 17.5228 2 12C2 6.47715 6.47715 2 12 2C17.5228 2 22 6.47715 22 12Z"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        >
        En attente
      </button>
      <button
        class="tab"
        class:active={activeTab === "history"}
        on:click={() => switchTab("history")}
      >
        <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
          ><path
            d="M3 12H21M3 6H21M3 18H14"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        >
        Historique des validations
      </button>
    </div>
  </div>

  {#if status}
    <div class="status-message {statusType}">{status}</div>
  {/if}

  {#if activeTab === "pending"}
    <!-- DASHBOARD HEADER -->
    <div class="dashboard-header">
      <div class="dashboard-filters">
        <div class="filter-group filter-search">
          <label class="filter-label">Recherche</label>
          <div class="search-control">
            <input
              class="meta-input"
              bind:value={searchQuery}
              placeholder="Structure, opération, site, type..."
              aria-label="Rechercher dans les dépôts"
            />
            <button
              type="button"
              class="btn btn-primary search-btn"
              on:click={applySearch}
            >
              Rechercher
            </button>
          </div>
        </div>
        <div class="filters-row">
          <div class="filter-group">
            <label class="filter-label">Type de structure</label>
            <div class="meta-select-wrapper">
              <select
                class="meta-select"
                bind:value={filterType}
                aria-label="Filtrer par type de structure"
              >
                <option value="">Tous</option>
                {#each presets.structure_types as t}
                  <option value={t}>{t}</option>
                {/each}
              </select>
            </div>
          </div>
          <div class="filter-group">
            <label class="filter-label">Opération</label>
            <div class="meta-select-wrapper">
              <select
                class="meta-select"
                bind:value={filterOperation}
                aria-label="Filtrer par opération"
              >
                <option value="">Toutes</option>
                {#each operationOptions as op}
                  <option value={op}>{op}</option>
                {/each}
              </select>
            </div>
          </div>
          <div class="filter-group">
            <label class="filter-label">Modification</label>
            <div class="meta-select-wrapper">
              <select
                class="meta-select"
                bind:value={filterRevision}
                aria-label="Filtrer par tag révision"
              >
                <option value="all">Tous</option>
                <option value="tagged">À modifier</option>
                <option value="untagged">Sans tag</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}

  {#if activeTab === "pending" && showPreviewModal && selectedItem}
    <div class="modal-overlay" on:click={() => (showPreviewModal = false)}>
      <div class="modal preview-modal" class:editing-mode={isEditing} on:click|stopPropagation>
        <div class="modal-header">
          <div class="modal-heading">
            <h2>Aperçu du dépôt</h2>
            <p class="modal-subtitle">
              {selectedItem.structure_folder} · {metadata?.operation.code ||
                selectedItem.operation_folder}
            </p>
          </div>
          <div class="modal-header-actions">
            <button
              class="btn btn-secondary modal-header-btn"
              on:click={() => selectedItem && revealInExplorer(selectedItem.path)}
              aria-label="Aller à l'emplacement"
              title="Aller à l'emplacement"
            >
              <svg viewBox="0 0 24 24" fill="none" width="15" height="15"
                ><path
                  d="M18 13V19C18 20.1046 17.1046 21 16 21H5C3.89543 21 3 20.1046 3 19V8C3 6.89543 3.89543 6 5 6H11"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /><path
                  d="M15 3H21V9"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /><path
                  d="M10 14L21 3"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /></svg
              >
              <span>Aller à l'emplacement</span>
            </button>
            <button
              class="modal-close"
              aria-label="Fermer la fenêtre d'aperçu"
              on:click={() => (showPreviewModal = false)}>×</button
            >
          </div>
        </div>
        <div class="modal-body" bind:this={previewBodyEl}>
          {#if !metadata}
            <p class="hint-text">Chargement...</p>
          {:else if isEditing && editMetadata}
            <div class="edit-grid">
              <div class="edit-section-title">Opération</div>
              <div class="field-col">
                <label class="meta-label">Code opération</label>
                <input
                  class="meta-input"
                  type="text"
                  class:input-error={!!editFieldErrors["operation.code"]}
                  bind:value={editMetadata.operation.code}
                />
                {#if editFieldErrors["operation.code"]}
                  <span class="field-error">{editFieldErrors["operation.code"]}</span>
                {/if}
              </div>
              <div class="field-col">
                <label class="meta-label">Site</label>
                <div class:combo-error={!!editFieldErrors["operation.site"]}>
                  <ComboInput
                    bind:value={editMetadata.operation.site}
                    options={presets.sites}
                  />
                </div>
                {#if editFieldErrors["operation.site"]}
                  <span class="field-error">{editFieldErrors["operation.site"]}</span>
                {/if}
              </div>
              <div class="field-col">
                <label class="meta-label">Type d'opération</label>
                <ComboInput
                  bind:value={editMetadata.operation.op_type}
                  options={presets.operation_types}
                />
              </div>
              <div class="field-col">
                <label class="meta-label">Responsable</label>
                <ComboInput
                  bind:value={editMetadata.operation.responsable}
                  options={presets.responsables}
                />
              </div>

              <div class="edit-section-title">Structure</div>
              <div class="field-col">
                <label class="meta-label">Identifiant</label>
                <input
                  class="meta-input"
                  type="text"
                  class:input-error={!!editFieldErrors["structure.id"]}
                  bind:value={editMetadata.structure.id}
                />
                {#if editFieldErrors["structure.id"]}
                  <span class="field-error">{editFieldErrors["structure.id"]}</span>
                {/if}
              </div>
              <div class="field-col">
                <label class="meta-label">Type de structure</label>
                <ComboInput
                  bind:value={editMetadata.structure.st_type}
                  options={presets.structure_types}
                />
              </div>
              <div class="field-col">
                <label class="meta-label">Auteur du modèle</label>
                <ComboInput
                  bind:value={editMetadata.structure.model_author}
                  options={presets.model_authors}
                />
              </div>
              <div class="field-col">
                <label class="meta-label">Déposant</label>
                <ComboInput
                  bind:value={editMetadata.structure.depositor}
                  options={presets.depositors}
                />
              </div>
              <div class="field-col">
                <label class="meta-label">Nombre de photos</label>
                <input
                  class="meta-input"
                  type="text"
                  bind:value={editMetadata.structure.photos_count}
                />
              </div>
              <div class="field-col">
                <label class="meta-label">Nombre de polygones</label>
                <input
                  class="meta-input auto-calculated-input"
                  type="text"
                  bind:value={editMetadata.structure.faces_count}
                  readonly
                  title="Champ calculé automatiquement à partir du modèle 3D"
                />
                <span class="field-hint">Calculé automatiquement</span>
              </div>
              <div class="field-col field-col-full">
                <label class="meta-label">Logiciels</label>
                <div class="software-chips">
                  {#each presets.software_types as sw}
                    <button
                      type="button"
                      class="software-chip"
                      class:selected={selectedSoftware.includes(sw)}
                      on:click={() => toggleSoftware(sw)}
                    >
                      {sw}
                    </button>
                  {/each}
                </div>
              </div>
              <div class="field-col field-col-full">
                <label class="meta-label">Description</label>
                <textarea
                  class="meta-input"
                  bind:value={editMetadata.structure.description}
                  rows="2"
                  style="min-height: 50px"
                ></textarea>
              </div>
            </div>
          {:else}
            <div class="meta-tools">
              <button class="btn btn-outline modal-btn" on:click={openEditModal}>
                <svg viewBox="0 0 24 24" fill="none" width="15" height="15"
                  ><path
                    d="M11 5H6C4.89543 5 4 5.89543 4 7V18C4 19.1046 4.89543 20 6 20H18C19.1046 20 20 19.1046 20 18V13"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  /><path
                    d="M17.5 2.5C18.3284 1.67157 19.6716 1.67157 20.5 2.5C21.3284 3.32843 21.3284 4.67157 20.5 5.5L12 14L8 15L9 11L17.5 2.5Z"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  /></svg
                >
                Éditer les métadonnées
              </button>
            </div>
            <div class="meta-minimal">
              <div class="meta-minimal-group">
                <div class="meta-minimal-title">Opération</div>
                <div class="meta-minimal-grid">
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Code opération</span>
                    <span class="meta-minimal-value">{metadata.operation.code}</span>
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Site</span>
                    <span class="meta-minimal-value">{metadata.operation.site}</span>
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Type d'opération</span>
                    <span class="meta-minimal-value"
                      >{metadata.operation.op_type || "—"}</span
                    >
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Responsable</span>
                    <span class="meta-minimal-value"
                      >{metadata.operation.responsable || "—"}</span
                    >
                  </div>
                </div>
              </div>

              <div class="meta-minimal-group">
                <div class="meta-minimal-title">Structure</div>
                <div class="meta-minimal-grid">
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Identifiant structure</span>
                    <span class="meta-minimal-value">{metadata.structure.id}</span>
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Type de structure</span>
                    <span class="meta-minimal-value">{metadata.structure.st_type}</span>
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Auteur du modèle</span>
                    <span class="meta-minimal-value"
                      >{metadata.structure.model_author || "—"}</span
                    >
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Déposant</span>
                    <span class="meta-minimal-value"
                      >{metadata.structure.depositor || "—"}</span
                    >
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Logiciels</span>
                    <span class="meta-minimal-value"
                      >{metadata.structure.software || "—"}</span
                    >
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Nombre de photos</span>
                    <span class="meta-minimal-value"
                      >{metadata.structure.photos_count || "—"}</span
                    >
                  </div>
                  <div class="meta-minimal-line">
                    <span class="meta-minimal-label">Nombre de polygones</span>
                    <span class="meta-minimal-value"
                      >{metadata.structure.faces_count || "—"}</span
                    >
                  </div>
                  <div class="meta-minimal-line meta-minimal-line-full">
                    <span class="meta-minimal-label">Description</span>
                    <span class="meta-minimal-value"
                      >{metadata.structure.description || "—"}</span
                    >
                  </div>
                </div>
              </div>
            </div>

            {#if selectedItem}
              {@const compact = buildCompactPreview(selectedItem.path)}
              <div class="preview-compact">
                <div class="preview-compact-files">
                  <span class="compact-pill" class:present={compact.models > 0}
                    >3D {compact.models}</span
                  >
                  <span class="compact-pill" class:present={compact.orthos > 0}
                    >Ortho {compact.orthos}</span
                  >
                  <span class="compact-pill" class:present={compact.photos > 0}
                    >Photos {compact.photos}</span
                  >
                  <span class="compact-pill" class:present={compact.work > 0}
                    >Travail {compact.work}</span
                  >
                </div>
                <div class="preview-compact-status">
                  <span class="compact-pill compact-pill-error"
                    >Error {compact.errorCount}</span
                  >
                  <span class="compact-pill compact-pill-warning"
                    >Warning {compact.warningCount}</span
                  >
                  <button
                    type="button"
                    class="compact-more"
                    on:click={() => (showCompactDetails = !showCompactDetails)}
                  >
                    {showCompactDetails ? "Voir moins" : "Voir plus"}
                  </button>
                </div>
              </div>
              {#if showCompactDetails}
                <div class="preview-compact-details">
                  {#if compact.errors.length > 0}
                    <p class="compact-details-title error">Erreurs</p>
                    <ul class="compact-details-list">
                      {#each compact.errors as line}
                        <li>{line}</li>
                      {/each}
                    </ul>
                  {/if}
                  {#if compact.warnings.length > 0}
                    <p class="compact-details-title warning">Warnings</p>
                    <ul class="compact-details-list">
                      {#each compact.warnings as line}
                        <li>{line}</li>
                      {/each}
                    </ul>
                  {/if}
                  {#if compact.errors.length === 0 && compact.warnings.length === 0}
                    <p class="compact-details-empty">Aucune anomalie détectée.</p>
                  {/if}
                </div>
              {/if}
            {/if}
          {/if}
        </div>
        <div class="modal-actions preview-actions">
          {#if isEditing}
            <button class="btn btn-secondary modal-btn" on:click={cancelEdit}>
              Annuler
            </button>
            <button
              class="btn btn-primary modal-btn"
              on:click={saveMetadata}
              disabled={processing || !editMetadataValid}
            >
              {processing ? "Enregistrement..." : "Enregistrer"}
            </button>
          {:else}
            {#if activeTab === "pending"}
              <button
                class="btn btn-warning modal-btn modal-btn-review"
                on:click={toggleRevisionTag}
                disabled={processing}
              >
                {processing
                  ? "Mise à jour..."
                  : revisionTags[selectedItem.path]?.tagged
                    ? "Retirer modification"
                    : "À modifier"}
              </button>
              <button
                class="btn btn-primary modal-btn modal-btn-archive"
                on:click={openArchiveConfirm}
                disabled={processing}
              >
                {processing ? "Archivage..." : "Archiver"}
              </button>
            {/if}
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if showArchiveConfirm && selectedItem}
    {@const compact = buildCompactPreview(selectedItem.path)}
    <div class="modal-overlay" on:click={() => (showArchiveConfirm = false)}>
      <div class="modal archive-confirm-modal" on:click|stopPropagation>
        <div class="modal-header">
          <div class="modal-heading">
            <h2>Confirmer l'archivage</h2>
            <p class="modal-subtitle">{selectedItem.structure_folder}</p>
          </div>
          <button
            class="modal-close"
            aria-label="Fermer la confirmation"
            on:click={() => (showArchiveConfirm = false)}>×</button
          >
        </div>
        <div class="modal-body">
          <p class="confirm-text">Êtes-vous sûr de vouloir archiver ce dépôt ?</p>
          {#if metadata}
            <div class="confirm-meta">
              <span><strong>Structure:</strong> {metadata.structure.id}</span>
              <span><strong>Opération:</strong> {metadata.operation.code}</span>
              <span><strong>Site:</strong> {metadata.operation.site || "—"}</span>
              <span><strong>Type:</strong> {metadata.structure.st_type || "—"}</span>
            </div>
          {/if}
          <div class="preview-compact">
            <div class="preview-compact-files">
              <span class="compact-pill" class:present={compact.models > 0}
                >3D {compact.models}</span
              >
              <span class="compact-pill" class:present={compact.orthos > 0}
                >Ortho {compact.orthos}</span
              >
              <span class="compact-pill" class:present={compact.photos > 0}
                >Photos {compact.photos}</span
              >
              <span class="compact-pill" class:present={compact.work > 0}
                >Travail {compact.work}</span
              >
            </div>
            <div class="preview-compact-status">
              <span class="compact-pill compact-pill-error"
                >Error {compact.errorCount}</span
              >
              <span class="compact-pill compact-pill-warning"
                >Warning {compact.warningCount}</span
              >
            </div>
          </div>
          <p class="confirm-state">
            {compact.errorCount === 0 && compact.warningCount === 0
              ? "Tous les indicateurs sont au vert."
              : compact.errorCount === 0
                ? "Archivage possible avec warnings."
                : "Des erreurs sont présentes, vérifiez avant d'archiver."}
          </p>
          {#if compact.errorCount > 0 || compact.warningCount > 0}
            <div class="confirm-issues">
              {#if compact.errorCount > 0}
                <p class="confirm-issues-title error">Erreurs ({compact.errorCount})</p>
                <ul class="confirm-issues-list">
                  {#each compact.errors.slice(0, 3) as issue}
                    <li>{issue}</li>
                  {/each}
                </ul>
              {/if}
              {#if compact.warningCount > 0}
                <p class="confirm-issues-title warning">Warnings ({compact.warningCount})</p>
                <ul class="confirm-issues-list">
                  {#each compact.warnings.slice(0, 3) as issue}
                    <li>{issue}</li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        </div>
        <div class="modal-actions">
          <button
            class="btn btn-secondary modal-btn"
            on:click={() => (showArchiveConfirm = false)}
          >
            Annuler
          </button>
          <button
            class="btn btn-primary modal-btn modal-btn-archive"
            on:click={archiveNow}
            disabled={processing}
          >
            {processing ? "Archivage..." : "Confirmer l'archivage"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if activeTab === "pending"}
    <!-- GRID -->
    <section class="dashboard-content">
      {#if loading}
        <p class="hint-text">Chargement...</p>
      {:else if filteredItems.length === 0}
        <p class="hint-text">Aucune structure correspondant aux filtres.</p>
      {:else}
        <div class="dashboard-grid">
          {#each filteredItems as item}
            <DepositCard
              title={item.structure_folder}
              operationCode={listDetails[item.path]?.metadata?.operation.code ||
                item.operation_folder}
              operationSite={listDetails[item.path]?.metadata?.operation.site ||
                "—"}
              structureType={listDetails[item.path]?.metadata?.structure
                .st_type || "—"}
              hasModel={(listDetails[item.path]?.models?.length || 0) > 0}
              hasPhotos={(listDetails[item.path]?.photos?.length || 0) > 0}
              hasOrtho={(listDetails[item.path]?.orthos?.length || 0) > 0}
              hasWork={(listDetails[item.path]?.work_files?.length || 0) > 0}
              revisionTagged={!!revisionTags[item.path]?.tagged}
              active={selectedItem?.path === item.path}
              on:click={() => selectItem(item)}
            />
          {/each}
        </div>
      {/if}
    </section>
  {:else}
    <section class="history-content">
      <div class="history-card">
        <div class="history-toolbar">
          <button
            type="button"
            class="btn btn-secondary btn-sm"
            on:click={resetHistory}
            disabled={resettingHistory}
          >
            {resettingHistory ? "Réinitialisation..." : "Reset historique"}
          </button>
        </div>
        {#if loadingHistory}
          <p class="hint-text">Chargement de l'historique...</p>
        {:else if historyEntries.length === 0}
          <p class="hint-text">Aucune validation récente.</p>
        {:else}
          <div class="history-table-wrap">
            <table class="history-table">
              <thead>
                <tr>
                  <th>Action</th>
                  <th>Structure</th>
                  <th>Utilisateur</th>
                  <th>Date</th>
                </tr>
              </thead>
              <tbody>
                {#each historyEntries as entry}
                  <tr>
                    <td>
                      <span
                        class="history-badge"
                        class:history-badge-warning={entry.action ===
                          "overridearchive"}
                      >
                        {actionLabel(entry.action)}
                      </span>
                    </td>
                    <td class="cell-bold">{structureName(entry.structure_path)}</td>
                    <td>{entry.user || "—"}</td>
                    <td class="cell-date">{formatTimestamp(entry.timestamp)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    </section>
  {/if}

  <!-- ETAPE 2 : METADONNÉES -->
</main>

<style>
  .validation-page {
    max-width: 1400px;
    margin: 0 auto;
    padding: var(--spacing-2xl);
  }

  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .page-header h1 {
    font-size: var(--font-size-xl);
    color: var(--color-neutral-900);
    margin: 0;
    font-weight: 700;
  }

  .header-subtitle {
    color: var(--color-neutral-600);
    font-size: var(--font-size-sm);
    margin: var(--spacing-xs) 0 0;
  }

  .tabs-bar {
    margin-bottom: var(--spacing-xl);
    border-bottom: 1px solid var(--color-neutral-400);
  }

  .tabs {
    display: flex;
    gap: 0;
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-xl);
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
    font-family: var(--font-family);
    color: var(--color-neutral-700);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: all 0.2s;
  }

  .tab:hover {
    color: var(--color-neutral-900);
  }

  .tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
    font-weight: 600;
  }

  .tab svg {
    opacity: 0.6;
  }

  .tab.active svg {
    opacity: 1;
  }

  /* ===== DASHBOARD HEADER ===== */
  .dashboard-header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: var(--spacing-md);
    padding: var(--spacing-lg) var(--spacing-md);
    border-bottom: 1px solid var(--color-neutral-300);
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-md);
  }

  .dashboard-filters {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    align-items: stretch;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    min-width: 0;
  }

  .filter-search {
    min-width: 0;
  }

  .filters-row {
    display: flex;
    flex-wrap: nowrap;
    gap: 8px;
    align-items: end;
    width: fit-content;
    max-width: 100%;
    align-self: flex-start;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
    padding-bottom: 2px;
  }

  .search-control {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    min-width: 0;
  }

  .search-control .meta-input {
    flex: 1;
    min-width: 0;
  }

  .search-btn {
    white-space: nowrap;
    min-height: 42px;
    padding-inline: var(--spacing-lg);
  }

  .filters-row .filter-group {
    flex: 0 0 clamp(150px, 16vw, 210px);
    width: clamp(150px, 16vw, 210px);
    gap: 4px;
  }

  .filters-row .meta-select {
    min-height: 34px;
    padding-top: 5px;
    padding-bottom: 5px;
    font-size: var(--font-size-sm);
  }

  .filters-row .meta-select-wrapper::after {
    right: 10px;
    width: 12px;
    height: 12px;
  }

  .filter-label {
    font-size: 0.69rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--color-neutral-600);
    font-weight: 700;
  }

  .pending-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 10px 14px;
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--shadow-sm);
  }

  .pending-count-wrap {
    width: 36px;
    height: 36px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--color-primary);
  }

  .pending-count {
    font-size: var(--font-size-base);
    font-weight: 900;
    color: var(--color-neutral-100);
    line-height: 1;
  }

  .pending-text {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }

  .pending-label {
    font-size: var(--font-size-sm);
    font-weight: 700;
    color: var(--color-neutral-800);
  }

  .pending-subtitle {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-600);
  }

  .dashboard-content {
    padding: var(--spacing-xl) var(--spacing-md);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 320px));
    gap: var(--spacing-md);
    justify-content: start;
  }

  .hint-text {
    color: var(--color-neutral-500);
    font-size: var(--font-size-sm);
    font-style: italic;
  }

  .history-content {
    padding: var(--spacing-xl) var(--spacing-md);
  }

  .history-card {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-lg);
  }

  .history-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--spacing-md);
  }

  .btn-sm {
    min-height: 34px;
    padding: 6px 12px;
    font-size: var(--font-size-xs);
  }

  .history-table-wrap {
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    overflow: hidden;
  }

  .history-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--font-size-sm);
  }

  .history-table th {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-neutral-200);
    font-size: var(--font-size-xs);
    text-align: left;
    white-space: nowrap;
  }

  .history-table td {
    padding: var(--spacing-sm) var(--spacing-md);
    border-top: 1px solid var(--color-neutral-300);
  }

  .history-badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: var(--border-radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 700;
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .history-badge-warning {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }

  .cell-bold {
    font-weight: 700;
  }

  .cell-date {
    white-space: nowrap;
  }

  /* ===== LOADING ===== */
  .loading-indicator {
    display: flex;
    justify-content: center;
    padding: var(--spacing-2xl);
  }

  .loader {
    width: 28px;
    height: 28px;
    border: 3px solid var(--color-neutral-300);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .btn-loader {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  /* ===== META MINIMAL (MODAL) ===== */
  .meta-minimal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: 2px 0;
  }

  .meta-minimal-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .meta-minimal-title {
    margin: 0;
    font-size: var(--font-size-xs);
    font-weight: 700;
    color: var(--color-primary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-neutral-300);
  }

  .meta-minimal-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px 20px;
  }

  .meta-minimal-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 4px 0;
    border-bottom: 1px dashed rgba(148, 163, 184, 0.35);
    min-width: 0;
  }

  .meta-minimal-line-full {
    grid-column: 1 / -1;
  }

  .meta-minimal-label {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--color-neutral-600);
    text-transform: uppercase;
    letter-spacing: 0.02em;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .meta-minimal-value {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-neutral-900);
    text-align: right;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  /* ===== MODAL ===== */
  .modal-overlay {
    position: fixed;
    inset: var(--app-topnav-height) 0 0 0;
    min-height: calc(100dvh - var(--app-topnav-height));
    background: rgba(15, 23, 42, 0.42);
    backdrop-filter: blur(5px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: var(--spacing-xl);
  }

  .modal {
    width: 100%;
    max-width: min(1100px, calc(100vw - 2rem));
    max-height: calc(100dvh - var(--app-topnav-height) - 2rem);
    overflow: hidden;
    background: var(--color-neutral-100);
    border-radius: 18px;
    border: 1px solid var(--color-neutral-400);
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.24);
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .preview-modal {
    max-width: min(980px, calc(100vw - 2rem));
  }

  .preview-modal.editing-mode {
    max-width: min(760px, calc(100vw - 2rem));
    max-height: min(calc(100dvh - var(--app-topnav-height) - 2rem), 700px);
  }

  .preview-modal.editing-mode .modal-body {
    padding-top: var(--spacing-md);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-lg) var(--spacing-xl);
    border-bottom: 1px solid var(--color-neutral-300);
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.96) 0%,
      rgba(246, 249, 253, 0.96) 100%
    );
  }

  .modal-heading {
    min-width: 0;
  }

  .modal-header-actions {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .modal-header-btn {
    min-height: 36px;
    padding: 0 12px;
    border-radius: 10px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    white-space: nowrap;
  }

  .modal-subtitle {
    margin: 4px 0 0;
    color: var(--color-neutral-600);
    font-size: var(--font-size-xs);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .modal-header h2 {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: 700;
    color: var(--color-neutral-900);
  }

  .modal-close {
    border: 1px solid var(--color-neutral-300);
    background: var(--color-neutral-100);
    color: var(--color-neutral-700);
    width: 34px;
    height: 34px;
    border-radius: 10px;
    cursor: pointer;
    font-size: 1.1rem;
    line-height: 1;
    transition: all 0.16s ease;
  }

  .modal-close:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
    transform: translateY(-1px);
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-height: 0;
    gap: var(--spacing-md);
    padding: var(--spacing-lg) var(--spacing-xl);
    overflow: auto;
  }

  .edit-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-md);
  }

  .edit-section-title {
    grid-column: 1 / -1;
    margin: 2px 0 0;
    font-size: var(--font-size-xs);
    font-weight: 700;
    color: var(--color-primary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--color-neutral-300);
  }

  .field-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .input-error {
    border-color: var(--color-error);
  }

  .combo-error :global(.combo-input) {
    border-color: var(--color-error);
  }

  .field-error {
    font-size: var(--font-size-xs);
    color: var(--color-error);
  }

  .auto-calculated-input {
    background: var(--color-neutral-200);
    color: var(--color-neutral-700);
    cursor: not-allowed;
  }

  .field-hint {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-600);
    margin-top: -2px;
  }

  .field-col-full {
    grid-column: 1 / -1;
  }

  .meta-tools {
    display: flex;
    justify-content: flex-start;
  }

  .preview-compact {
    margin-top: var(--spacing-xs);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 8px 10px;
    border: 1px solid var(--color-neutral-300);
    border-radius: 10px;
    background: var(--color-neutral-100);
  }

  .preview-compact-files {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .preview-compact-status {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }

  .compact-pill {
    font-size: 0.68rem;
    font-weight: 700;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid var(--color-neutral-300);
    color: var(--color-neutral-600);
    background: var(--color-neutral-200);
    line-height: 1.1;
  }

  .compact-pill.present {
    color: var(--color-success);
    background: var(--color-success-bg);
    border-color: var(--color-success-border);
  }

  .compact-pill-error {
    color: var(--color-error);
    background: var(--color-error-bg);
    border-color: var(--color-error-border);
  }

  .compact-pill-warning {
    color: var(--color-warning);
    background: var(--color-warning-bg);
    border-color: var(--color-warning-border);
  }

  .compact-more {
    border: none;
    background: transparent;
    color: var(--color-primary);
    font-size: 0.68rem;
    font-weight: 700;
    text-decoration: underline;
    cursor: pointer;
    padding: 2px 4px;
  }

  .preview-compact-details {
    margin-top: 6px;
    padding: 8px 10px;
    border: 1px solid var(--color-neutral-300);
    border-radius: 10px;
    background: var(--color-neutral-100);
  }

  .compact-details-title {
    margin: 0 0 4px;
    font-size: 0.68rem;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .compact-details-title.error {
    color: var(--color-error);
  }

  .compact-details-title.warning {
    color: var(--color-warning);
  }

  .compact-details-list {
    margin: 0 0 6px 16px;
    padding: 0;
    font-size: var(--font-size-xs);
    color: var(--color-neutral-800);
  }

  .compact-details-empty {
    margin: 0;
    font-size: var(--font-size-xs);
    color: var(--color-success);
    font-weight: 600;
  }

  .software-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
  }

  .software-chip {
    padding: 6px 12px;
    border-radius: 999px;
    border: 1px solid var(--color-neutral-400);
    background: var(--color-neutral-100);
    color: var(--color-neutral-800);
    font-size: var(--font-size-xs);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .software-chip:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .software-chip.selected {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-neutral-100);
  }

  .modal-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    flex-wrap: wrap;
    padding: var(--spacing-md) var(--spacing-xl) var(--spacing-lg);
    border-top: 1px solid var(--color-neutral-300);
    background: var(--color-neutral-100);
  }

  .preview-actions {
    justify-content: flex-end;
  }

  .modal-btn {
    min-width: 170px;
    min-height: 38px;
    border-radius: 10px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .modal-btn-archive {
    min-width: 190px;
    box-shadow: 0 4px 14px rgba(30, 58, 95, 0.24);
  }

  .btn-warning {
    background: var(--color-warning-bg);
    color: var(--color-warning);
    border: var(--border-width) solid var(--color-warning-border);
  }

  .btn-warning:hover:not(:disabled) {
    background: #fce8b2;
    border-color: #f4cc6a;
    transform: translateY(-1px);
  }

  .modal-btn-review {
    min-width: 170px;
  }

  .archive-confirm-modal {
    max-width: 640px;
  }

  .confirm-text {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
    font-weight: 600;
  }

  .confirm-meta {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 4px 12px;
    font-size: var(--font-size-xs);
    color: var(--color-neutral-700);
    padding: 6px 0;
  }

  .confirm-state {
    margin: 0;
    font-size: var(--font-size-xs);
    color: var(--color-neutral-600);
    font-weight: 600;
  }

  .confirm-issues {
    border: 1px solid var(--color-neutral-300);
    border-radius: 10px;
    padding: 8px 10px;
    background: var(--color-neutral-100);
  }

  .confirm-issues-title {
    margin: 0 0 4px;
    font-size: 0.72rem;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .confirm-issues-title.error {
    color: var(--color-error);
  }

  .confirm-issues-title.warning {
    color: var(--color-warning);
  }

  .confirm-issues-list {
    margin: 0 0 6px 16px;
    padding: 0;
    font-size: var(--font-size-xs);
    color: var(--color-neutral-800);
  }

  @media (max-height: 820px) {
    .modal-overlay {
      padding-top: var(--spacing-sm);
      padding-bottom: var(--spacing-sm);
    }

    .modal {
      max-height: calc(100dvh - var(--app-topnav-height) - 0.75rem);
    }
  }

  @media (max-width: 1024px) {
    .validation-page {
      padding: var(--spacing-xl);
    }

    .dashboard-header {
      align-items: stretch;
      flex-direction: column;
      gap: var(--spacing-md);
      padding: var(--spacing-md);
    }

    .page-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .dashboard-filters {
      width: 100%;
      gap: var(--spacing-sm);
    }

    .filters-row {
      gap: 8px;
    }
  }

  @media (max-width: 900px) {
    .dashboard-header {
      gap: var(--spacing-md);
    }

    .dashboard-filters {
      width: 100%;
      gap: var(--spacing-sm);
    }

    .filter-search {
      grid-column: auto;
    }

    .filter-group {
      min-width: 0;
      width: auto;
    }

    .filter-label {
      margin-bottom: 2px;
    }

    .dashboard-grid {
      grid-template-columns: repeat(auto-fit, minmax(220px, 300px));
      justify-content: start;
    }

    .modal-overlay {
      padding: var(--spacing-md);
      align-items: center;
    }

    .modal {
      max-height: calc(100dvh - var(--app-topnav-height) - 1.25rem);
    }

    .modal-header,
    .modal-body,
    .modal-actions {
      padding-left: var(--spacing-lg);
      padding-right: var(--spacing-lg);
    }

    .modal-header {
      align-items: flex-start;
      flex-wrap: wrap;
      gap: var(--spacing-sm);
    }

    .modal-heading {
      flex: 1 1 220px;
    }

    .modal-header-actions {
      flex-wrap: wrap;
      justify-content: flex-end;
    }

    .edit-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .modal-actions {
      justify-content: flex-end;
    }

    .modal-actions :global(.btn),
    .modal-actions .modal-btn {
      width: auto;
      min-width: 150px;
    }
  }

  @media (max-width: 640px) {
    .validation-page {
      padding: var(--spacing-md);
    }

    .dashboard-header {
      padding: var(--spacing-sm);
    }

    .dashboard-content {
      padding: var(--spacing-lg) var(--spacing-sm);
    }

    .history-content {
      padding: var(--spacing-lg) var(--spacing-sm);
    }

    .dashboard-filters {
      gap: var(--spacing-xs);
    }

    .filters-row {
      gap: 6px;
    }

    .dashboard-grid {
      grid-template-columns: 1fr;
      justify-content: stretch;
    }

    .search-btn {
      min-height: 40px;
      padding-inline: var(--spacing-md);
    }

    .history-table th,
    .history-table td {
      padding: var(--spacing-xs) var(--spacing-sm);
    }

    .modal-actions :global(.btn),
    .modal-actions .modal-btn {
      width: auto;
      min-width: 120px;
    }

    .modal-header,
    .modal-body,
    .modal-actions {
      padding-left: var(--spacing-md);
      padding-right: var(--spacing-md);
    }

    .modal-header-btn {
      min-height: 34px;
      padding-inline: 10px;
      font-size: var(--font-size-xs);
    }
  }

  @media (max-width: 480px) {
    .validation-page {
      padding: var(--spacing-sm);
    }

    .dashboard-header {
      padding: var(--spacing-sm) var(--spacing-xs);
    }

    .dashboard-filters {
      gap: var(--spacing-xs);
    }

    .filters-row {
      gap: 6px;
    }

    .tab {
      padding: var(--spacing-sm) var(--spacing-md);
    }

    .search-control {
      flex-wrap: wrap;
    }

    .search-btn {
      width: 100%;
    }

    .modal-header-btn {
      width: 100%;
    }

    .meta-minimal-grid {
      grid-template-columns: 1fr;
    }

    .meta-minimal-value {
      text-align: left;
    }
  }

  @media (max-width: 560px) {
    .modal-overlay {
      padding: var(--spacing-xs);
    }

    .modal {
      max-width: calc(100vw - 0.5rem);
      max-height: calc(100dvh - var(--app-topnav-height) - 0.5rem);
    }

    .modal-actions {
      justify-content: stretch;
    }

    .modal-actions :global(.btn),
    .modal-actions .modal-btn {
      width: 100%;
      min-width: 0;
    }

    .edit-grid {
      grid-template-columns: 1fr;
    }

    .confirm-meta {
      grid-template-columns: 1fr;
    }
  }
</style>
