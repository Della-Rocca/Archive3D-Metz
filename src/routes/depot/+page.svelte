<script lang="ts">
  import {
    getMetadataPresets,
    countModelPolygons,
    depositStructure,
  } from "$lib/api/deposit";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount, onDestroy } from "svelte";
  import { can } from "$lib/stores/auth";
  import MetadataEditor from "$lib/components/MetadataEditor.svelte";
  import ComboInput from "$lib/components/ComboInput.svelte";
  import DepositStepper from "$lib/components/domain/depot/DepositStepper.svelte";
  import FileDropZone from "$lib/components/domain/depot/FileDropZone.svelte";
  import DepositStepOperation from "$lib/components/domain/depot/DepositStepOperation.svelte";
  import DepositStepStructure from "$lib/components/domain/depot/DepositStepStructure.svelte";
  import DepositStepRecap from "$lib/components/domain/depot/DepositStepRecap.svelte";
  import { filename } from "$lib/utils/file";

  // --- Types ---
  import type {
    OperationMeta,
    StructureMeta,
    Presets,
  } from "$lib/types/deposit";

  // --- Stepper ---
  const STEPS = [
    { id: 1, label: "Opération", desc: "Rattachement" },
    { id: 2, label: "Structure", desc: "Métadonnées" },
    { id: 3, label: "Fichiers", desc: "Modèles & données" },
    { id: 4, label: "Récapitulatif", desc: "Vérification" },
  ];
  let currentStep = 1;

  // --- State ---
  let operation: OperationMeta = {
    code: "",
    site: "",
    op_type: "",
    responsable: "",
  };
  let structure: StructureMeta = {
    id: "",
    st_type: "",
    description: "",
    model_author: "",
    depositor: "",
    photos_count: "",
    faces_count: "",
    software: "",
  };

  let modelFiles: string[] = [];
  let orthoFiles: string[] = [];
  let photoFiles: string[] = [];
  let workFiles: string[] = [];

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

  let status = "";
  let statusType: "success" | "error" | "warning" | "" = "";
  let loading = false;
  let countingPolygons = false;
  let showMetadataEditor = false;

  // --- Validation inline (touched fields) ---
  let touched: Record<string, boolean> = {};

  function touch(field: string) {
    touched[field] = true;
    touched = touched;
  }

  // Erreurs par champ
  $: fieldErrors = {
    "operation.code":
      touched["operation.code"] && !operation.code.trim()
        ? "Champ obligatoire"
        : "",
    "structure.id":
      touched["structure.id"] && !structure.id.trim()
        ? "Champ obligatoire"
        : "",
    "structure.st_type":
      touched["structure.st_type"] && !structure.st_type.trim()
        ? "Champ obligatoire"
        : "",
    modelFiles:
      touched["modelFiles"] && modelFiles.length === 0
        ? "Au moins un modèle 3D requis"
        : "",
  };

  // Validation par étape
  $: step1Valid = !!operation.code.trim() && !!operation.site.trim();
  $: step2Valid = !!structure.id.trim() && !!structure.st_type.trim();
  $: step3Valid = modelFiles.length > 0;
  $: allValid = step1Valid && step2Valid && step3Valid;

  function goToStep(step: number) {
    // On peut revenir en arrière librement, avancer seulement si étape courante valide
    if (step < currentStep) {
      currentStep = step;
      return;
    }
    if (step === currentStep + 1 && canAdvance()) {
      currentStep = step;
    }
  }

  function canAdvance(): boolean {
    if (currentStep === 1) return step1Valid;
    if (currentStep === 2) return step2Valid;
    if (currentStep === 3) return step3Valid;
    return false;
  }

  function nextStep() {
    // Marquer les champs obligatoires de l'étape comme touched
    if (currentStep === 1) {
      touch("operation.code");
    }
    if (currentStep === 2) {
      touch("structure.id");
      touch("structure.st_type");
    }
    if (currentStep === 3) {
      touch("modelFiles");
    }
    if (canAdvance() && currentStep < 4) currentStep++;
  }

  function prevStep() {
    if (currentStep > 1) currentStep--;
  }

  // Synchroniser le nombre de photos depuis les fichiers sélectionnés
  $: structure.photos_count =
    photoFiles.length > 0 ? photoFiles.length.toString() : "";



  // Permission admin pour éditer les métadonnées
  const canEditMetadataStore = can("canEditMetadata");
  $: canEditMetadata = $canEditMetadataStore;

  // --- Load presets ---
  async function loadPresets() {
    try {
      presets = await getMetadataPresets();
      presets.software_types = presets.software_types.sort();
    } catch (e) {
      console.error("Erreur chargement presets:", e);
    }
  }

  onMount(() => {
    loadPresets();
    setupDragDrop();
  });

  function handleMetadataSaved() {
    loadPresets();
    showMetadataEditor = false;
  }

  // --- Fill operation from preset ---


  // --- File pickers ---
  async function pickFiles(
    title: string,
    filters?: { name: string; extensions: string[] }[],
    multiple = true,
  ): Promise<string[]> {
    const result = await open({
      title,
      multiple,
      filters: filters ?? [],
    });
    if (!result) return [];
    return Array.isArray(result) ? result : [result];
  }

  // --- Duplicate detection ---
  let duplicateWarning = "";
  let dropError = "";

  const DROP_LABELS: Record<"model" | "ortho" | "photo" | "work", string> = {
    model: "Modèles 3D",
    ortho: "Orthomosaïques",
    photo: "Photos",
    work: "Fichiers de travail",
  };

  const DROP_EXTENSIONS: Record<
    "model" | "ortho" | "photo" | "work",
    string[]
  > = {
    model: ["glb", "gltf", "obj"],
    ortho: ["tif", "tiff", "png", "jpg", "jpeg"],
    photo: [
      "jpg",
      "jpeg",
      "png",
      "tiff",
      "tif",
      "raw",
      "cr2",
      "nef",
      "arw",
      "dng",
    ],
    work: [],
  };

  function extensionOf(path: string): string {
    const name = filename(path);
    const parts = name.split(".");
    if (parts.length < 2) return "";
    return parts[parts.length - 1].toLowerCase();
  }

  function formatExtensions(exts: string[]): string {
    if (exts.length === 0) return "tous formats";
    return exts.map((e) => `.${e}`).join(", ");
  }

  function showDropError(zone: "model" | "ortho" | "photo" | "work") {
    const exts = DROP_EXTENSIONS[zone];
    dropError = `Fichiers non autorisés pour ${DROP_LABELS[zone]}. Extensions acceptées : ${formatExtensions(exts)}.`;
    setTimeout(() => {
      dropError = "";
    }, 5000);
  }

  function addFilesWithDedup(
    existing: string[],
    newFiles: string[],
  ): { merged: string[]; dupes: string[] } {
    const existingNames = new Set(existing.map((f) => filename(f)));
    const dupes: string[] = [];
    const toAdd: string[] = [];
    for (const f of newFiles) {
      const name = filename(f);
      if (existingNames.has(name)) {
        dupes.push(name);
      } else {
        existingNames.add(name);
        toAdd.push(f);
      }
    }
    return { merged: [...existing, ...toAdd], dupes };
  }

  function showDupeWarning(dupes: string[]) {
    if (dupes.length > 0) {
      duplicateWarning = `Doublon${dupes.length > 1 ? "s" : ""} ignoré${dupes.length > 1 ? "s" : ""} : ${dupes.join(", ")}`;
      setTimeout(() => {
        duplicateWarning = "";
      }, 4000);
    }
  }

  async function pickModelFiles() {
    const files = await pickFiles("Sélectionner les modèles 3D", [
      { name: "Modèles 3D", extensions: ["glb", "gltf", "obj"] },
    ], false);
    if (files.length > 0) {
      if (modelFiles.length > 0) {
        dropError = "Un seul modèle 3D est autorisé. Supprimez le modèle actuel avant d'en ajouter un autre.";
        setTimeout(() => {
          dropError = "";
        }, 5000);
        return;
      }
      const firstModel = files[0];
      const { merged, dupes } = addFilesWithDedup(modelFiles, [firstModel]);
      modelFiles = merged.slice(0, 1);
      showDupeWarning(dupes);
      await countPolygons();
    }
  }

  async function pickOrthoFiles() {
    const files = await pickFiles("Sélectionner les orthomosaïques", [
      { name: "Images", extensions: ["tif", "tiff", "png", "jpg", "jpeg"] },
    ]);
    if (files.length > 0) {
      const { merged, dupes } = addFilesWithDedup(orthoFiles, files);
      orthoFiles = merged;
      showDupeWarning(dupes);
    }
  }

  async function pickPhotoFiles() {
    const files = await pickFiles("Sélectionner les photos", [
      {
        name: "Photos",
        extensions: [
          "jpg",
          "jpeg",
          "png",
          "tiff",
          "tif",
          "raw",
          "cr2",
          "nef",
          "arw",
          "dng",
        ],
      },
    ]);
    if (files.length > 0) {
      const { merged, dupes } = addFilesWithDedup(photoFiles, files);
      photoFiles = merged;
      showDupeWarning(dupes);
    }
  }

  async function pickWorkFiles() {
    const files = await pickFiles("Sélectionner les fichiers de travail");
    if (files.length > 0) {
      const { merged, dupes } = addFilesWithDedup(workFiles, files);
      workFiles = merged;
      showDupeWarning(dupes);
    }
  }

  async function addModelFiles() {
    await pickModelFiles();
  }

  async function addOrthoFiles() {
    await pickOrthoFiles();
  }

  async function addPhotoFiles() {
    await pickPhotoFiles();
  }

  async function addWorkFiles() {
    await pickWorkFiles();
  }

  function removeFile(
    list: "model" | "ortho" | "photo" | "work",
    index: number,
  ) {
    if (list === "model") {
      modelFiles = modelFiles.filter((_, i) => i !== index);
      if (modelFiles.length === 0) {
        structure.faces_count = "";
      } else {
        void countPolygons();
      }
    }
    else if (list === "ortho")
      orthoFiles = orthoFiles.filter((_, i) => i !== index);
    else if (list === "photo")
      photoFiles = photoFiles.filter((_, i) => i !== index);
    else workFiles = workFiles.filter((_, i) => i !== index);
  }

  // --- Drag & Drop natif Tauri ---
  let dragOver: string | null = null;
  let unlistenDragDrop: (() => void) | null = null;
  const DROP_IDS = ["model", "ortho", "photo", "work"] as const;
  type DropId = (typeof DROP_IDS)[number];

  function resolveDropZoneFromPosition(pos: { x: number; y: number }): DropId | null {
    const dpr = window.devicePixelRatio || 1;
    const points: [number, number][] = [
      [pos.x, pos.y],
      [pos.x / dpr, pos.y / dpr],
    ];

    for (const [x, y] of points) {
      const elements = document.elementsFromPoint(x, y);
      for (const el of elements) {
        const card = el.closest("[data-drop]") as HTMLElement | null;
        const id = card?.dataset.drop as DropId | undefined;
        if (id && DROP_IDS.includes(id)) return id;
      }
    }
    return null;
  }

  async function setupDragDrop() {
    try {
      const webview = getCurrentWebview();
      unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
        if (event.payload.type === "over") {
          dragOver = resolveDropZoneFromPosition(event.payload.position);
        } else if (event.payload.type === "drop") {
          const paths = event.payload.paths;
          const dropZone =
            resolveDropZoneFromPosition(event.payload.position) ||
            (dragOver as DropId | null);

          if (dropZone && paths.length > 0) {
            await handleDroppedFiles(dropZone, paths);
          }
          dragOver = null;
        } else if (event.payload.type === "leave") {
          dragOver = null;
        }
      });
    } catch (e) {
      console.error("Drag-drop setup error:", e);
    }
  }

  async function handleDroppedFiles(
    zone: "model" | "ortho" | "photo" | "work",
    paths: string[],
  ) {
    const allowed = DROP_EXTENSIONS[zone];
    let validPaths = paths;
    if (allowed.length > 0) {
      validPaths = paths.filter((p) => allowed.includes(extensionOf(p)));
      if (validPaths.length === 0) {
        showDropError(zone);
        return;
      }
      if (validPaths.length < paths.length) {
        showDropError(zone);
      }
    }

    if (zone === "model") {
      if (modelFiles.length > 0) {
        dropError = "Un seul modèle 3D est autorisé. Supprimez le modèle actuel avant d'en ajouter un autre.";
        setTimeout(() => {
          dropError = "";
        }, 5000);
        return;
      }
      if (validPaths.length > 1) {
        dropError = "Un seul modèle 3D est autorisé. Seul le premier fichier a été conservé.";
        setTimeout(() => {
          dropError = "";
        }, 5000);
      }
      const { merged, dupes } = addFilesWithDedup(modelFiles, [validPaths[0]]);
      modelFiles = merged;
      showDupeWarning(dupes);
      await countPolygons();
    } else if (zone === "ortho") {
      const { merged, dupes } = addFilesWithDedup(orthoFiles, validPaths);
      orthoFiles = merged;
      showDupeWarning(dupes);
    } else if (zone === "photo") {
      const { merged, dupes } = addFilesWithDedup(photoFiles, validPaths);
      photoFiles = merged;
      showDupeWarning(dupes);
    } else {
      const { merged, dupes } = addFilesWithDedup(workFiles, validPaths);
      workFiles = merged;
      showDupeWarning(dupes);
    }
  }

  onDestroy(() => {
    if (unlistenDragDrop) unlistenDragDrop();
  });

  async function countPolygons() {
    const glbFile = modelFiles.find(
      (f) => f.endsWith(".glb") || f.endsWith(".gltf"),
    );
    if (!glbFile) {
      structure.faces_count = "";
      return;
    }
    countingPolygons = true;
    try {
      const count = await countModelPolygons(glbFile);
      structure.faces_count = count.toString();
    } catch (e) {
      console.error("Erreur comptage polygones:", e);
      structure.faces_count = "";
    } finally {
      countingPolygons = false;
    }
  }

  // --- Submit ---
  async function handleDeposit() {
    if (!allValid) return;
    loading = true;
    status = "";
    statusType = "";
    try {
      await depositStructure(
        { operation, structure },
        {
          model: modelFiles,
          ortho: orthoFiles,
          photo: photoFiles,
          work: workFiles,
        }
      );
      status = "Dépôt réussi ! Structure enregistrée dans l'espace de dépôt.";
      statusType = "success";
      resetForm();
    } catch (e) {
      status = `Erreur lors du dépôt : ${e}`;
      statusType = "error";
      currentStep = 4; // rester sur le recap pour voir l'erreur
    } finally {
      loading = false;
    }
  }

  function resetForm() {
    operation = { code: "", site: "", op_type: "", responsable: "" };
    structure = {
      id: "",
      st_type: "",
      description: "",
      model_author: "",
      depositor: "",
      photos_count: "",
      faces_count: "",
      software: "",
    };
    modelFiles = [];
    orthoFiles = [];
    photoFiles = [];
    workFiles = [];
    touched = {};
    currentStep = 1;
  }

  // --- Recap helpers ---
  $: filesCounts = {
    model: modelFiles.length,
    ortho: orthoFiles.length,
    photo: photoFiles.length,
    work: workFiles.length,
  };

  let recapIssues: string[] = [];
  $: {
    const issues: string[] = [];
    if (!operation.op_type) issues.push("Type d'opération");
    if (!operation.responsable) issues.push("Responsable d'opération");
    if (!structure.model_author) issues.push("Auteur du modèle");
    if (!structure.depositor) issues.push("Déposant");
    if (!structure.software) issues.push("Logiciels utilisés");
    if (!structure.description) issues.push("Description");
    recapIssues = issues;
  }
</script>

<main class="depot-page">
  {#if showMetadataEditor}
    <MetadataEditor
      on:close={() => (showMetadataEditor = false)}
      on:saved={handleMetadataSaved}
    />
  {:else}
    <!-- HEADER -->
    <div class="page-header">
      <div class="page-header-left">
        <h1>Dépôt de structure</h1>
        <p class="page-subtitle">
          Remplissez les métadonnées puis sélectionnez les fichiers à archiver.
        </p>
      </div>
      {#if canEditMetadata}
        <button
          class="btn btn-outline edit-meta-btn"
          on:click={() => (showMetadataEditor = true)}
        >
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
          Gérer les métadonnées
        </button>
      {/if}
    </div>

    {#if status}
      <div class="status-message {statusType}">{status}</div>
    {/if}

    {#if duplicateWarning}
      <div class="status-message warning">{duplicateWarning}</div>
    {/if}
    {#if dropError}
      <div class="status-message error">{dropError}</div>
    {/if}

    <!-- STEPPER -->
    <DepositStepper
      steps={STEPS}
      {currentStep}
      {step1Valid}
      {step2Valid}
      {step3Valid}
      on:stepClick={(e) => goToStep(e.detail)}
    />

    <!-- ======================== -->
    <!--   ÉTAPE 1 : OPÉRATION   -->
    <!-- ======================== -->
    {#if currentStep === 1}
      <DepositStepOperation
        bind:operation={operation}
        {presets}
        errors={fieldErrors}
      />
    {/if}

    <!-- ======================== -->
    <!--   ÉTAPE 2 : MÉTADONNÉES   -->
    {#if currentStep === 2}
      <DepositStepStructure
        bind:structure={structure}
        {presets}
        errors={fieldErrors}
        {countingPolygons}
        on:touch={(e) => touch(e.detail)}
      />
    {/if}

    <!-- ======================== -->
    <!--   ÉTAPE 3 : FICHIERS    -->
    <!-- ======================== -->
    {#if currentStep === 3}
      <section
        class="form-section section-files"
        style="animation: fadeIn 0.2s ease"
      >
        <div class="section-body">
          {#if fieldErrors["modelFiles"]}
            <div
              class="status-message error"
              style="margin-bottom: var(--spacing-lg)"
            >
              {fieldErrors["modelFiles"]}
            </div>
          {/if}
          <div class="file-cards">
            <!-- Modèles 3D -->
            <FileDropZone
              title="Modèles 3D"
              required={true}
              files={modelFiles}
              dropId="model"
              {dragOver}
              error={fieldErrors["modelFiles"]}
              on:add={addModelFiles}
              on:remove={(e) => removeFile("model", e.detail)}
            />

            <!-- Orthomosaïques -->
            <FileDropZone
              title="Orthophotographies"
              files={orthoFiles}
              dropId="ortho"
              {dragOver}
              on:add={addOrthoFiles}
              on:remove={(e) => removeFile("ortho", e.detail)}
            />

            <!-- Photos -->
            <FileDropZone
              title="Photos"
              files={photoFiles}
              dropId="photo"
              {dragOver}
              on:add={addPhotoFiles}
              on:remove={(e) => removeFile("photo", e.detail)}
            />

            <!-- Fichiers de travail -->
            <FileDropZone
              title="Fichiers de travail"
              files={workFiles}
              dropId="work"
              {dragOver}
              on:add={addWorkFiles}
              on:remove={(e) => removeFile("work", e.detail)}
            />
          </div>
        </div>
      </section>
    {/if}

    <!-- ============================ -->
    <!--   ÉTAPE 4 : RÉCAPITULATIF   -->
    <!-- ============================ -->
    <!--   ÉTAPE 4 : RÉCAPITULATIF   -->
    {#if currentStep === 4}
      <DepositStepRecap
        {operation}
        {structure}
        {filesCounts}
        {recapIssues}
        on:edit={(e) => goToStep(e.detail)}
      />
    {/if}

    <!-- BARRE D'ACTIONS (navigation stepper) -->
    <div class="form-actions">
      <div class="form-actions-left">
        {#if currentStep > 1}
          <button
            class="btn btn-secondary"
            on:click={prevStep}
            disabled={loading}
          >
            <svg viewBox="0 0 24 24" fill="none" width="14" height="14"
              ><path
                d="M15 18L9 12L15 6"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              /></svg
            >
            Précédent
          </button>
        {/if}
        <button
          class="btn btn-secondary"
          on:click={resetForm}
          disabled={loading}>Réinitialiser</button
        >
      </div>

      {#if currentStep < 4}
        <button class="btn btn-primary" on:click={nextStep} disabled={loading}>
          Suivant
          <svg viewBox="0 0 24 24" fill="none" width="14" height="14"
            ><path
              d="M9 18L15 12L9 6"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            /></svg
          >
        </button>
      {:else}
        <button
          class="btn btn-primary btn-lg"
          on:click={handleDeposit}
          disabled={loading || !allValid}
        >
          {#if loading}
            <span class="spinner"></span> Dépôt en cours...
          {:else}
            <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
              ><path
                d="M21 15V19C21 20.1046 20.1046 21 19 21H5C3.89543 21 3 20.1046 3 19V15M7 10L12 15M12 15L17 10M12 15V3"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              /></svg
            >
            Déposer la structure
          {/if}
        </button>
      {/if}
    </div>
  {/if}
</main>

<style>
  /* ===== LAYOUT ===== */
  main {
    padding: var(--spacing-2xl);
    max-width: 960px;
    margin: 0 auto;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ===== HEADER ===== */
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xl);
    padding-bottom: var(--spacing-lg);
    border-bottom: 1px solid var(--color-neutral-300);
  }
  .page-header h1 {
    font-size: var(--font-size-xl);
    color: var(--color-neutral-900);
    margin: 0;
    font-weight: 700;
  }
  .page-subtitle {
    color: var(--color-neutral-600);
    font-size: var(--font-size-sm);
    margin: 4px 0 0;
  }
  .edit-meta-btn {
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* ===== FORM SECTIONS ===== */
  .form-section {
    background: var(--color-neutral-100);
    border-radius: var(--border-radius-lg);
    border: 1px solid var(--color-neutral-300);
    margin-bottom: var(--spacing-xl);
    overflow: hidden;
  }
  .section-body {
    padding: var(--spacing-xl);
  }

  /* ===== FIELD GROUPS ===== */
  .field-group {
    margin-bottom: var(--spacing-xl);
  }
  .field-group:last-child {
    margin-bottom: 0;
  }
  .field-group-title {
    margin: 0 0 var(--spacing-md);
    font-size: var(--font-size-xs);
    font-weight: 700;
    color: var(--color-primary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding-bottom: var(--spacing-xs);
    border-bottom: 1px solid var(--color-neutral-300);
  }
  .field-row-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
  }
  .field-col {
    display: flex;
    flex-direction: column;
  }
  .desc-label {
    margin-top: var(--spacing-md) !important;
  }

  /* ===== VALIDATION INLINE ===== */
  .field-error {
    display: block;
    font-size: var(--font-size-xs);
    color: var(--color-error);
    margin-top: 3px;
    font-weight: 500;
  }

  /* ===== OPERATION SUMMARY ===== */
  .op-summary {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--spacing-sm);
    margin-top: var(--spacing-lg);
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--color-neutral-200);
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-neutral-300);
  }
  .op-summary-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .op-summary-label {
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--color-neutral-600);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .op-summary-value {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-neutral-900);
  }
  .op-summary-empty {
    border-style: dashed;
    background: var(--color-neutral-100);
  }
  .op-summary-placeholder {
    color: var(--color-neutral-500);
    font-weight: 400;
    font-style: italic;
  }
  .hint-text {
    color: var(--color-neutral-500);
    font-size: var(--font-size-sm);
    font-style: italic;
    margin: var(--spacing-md) 0 0;
  }

  /* ===== FICHIERS ===== */
  .file-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 320px));
    column-gap: var(--spacing-md);
    row-gap: var(--spacing-lg);
    grid-auto-rows: minmax(0, auto);
    justify-content: center;
    align-items: start;
  }
  .file-card {
    border: 2px dashed var(--color-neutral-400);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-lg);
    transition: all 0.2s;
    background: var(--color-neutral-100);
  }
  .file-card-filled {
    border-style: solid;
    border-color: var(--color-primary);
    background: rgba(30, 58, 95, 0.02);
  }
  .file-card-dragover {
    border-color: var(--color-secondary) !important;
    border-style: dashed !important;
    border-width: 2.5px !important;
    background: rgba(193, 122, 92, 0.06) !important;
    box-shadow:
      0 0 0 4px rgba(193, 122, 92, 0.12),
      0 4px 12px rgba(193, 122, 92, 0.08);
    transform: scale(1.01);
    animation: dropPulse 1.2s ease-in-out infinite;
  }

  @keyframes dropPulse {
    0%,
    100% {
      box-shadow:
        0 0 0 4px rgba(193, 122, 92, 0.12),
        0 4px 12px rgba(193, 122, 92, 0.08);
    }
    50% {
      box-shadow:
        0 0 0 6px rgba(193, 122, 92, 0.18),
        0 4px 16px rgba(193, 122, 92, 0.12);
    }
  }

  /* Drop hint compact visible meme quand des fichiers sont presents */
  .file-drop-hint {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm);
    margin-top: var(--spacing-sm);
    border-top: 1px dashed var(--color-neutral-400);
    color: var(--color-neutral-500);
    font-size: var(--font-size-xs);
    transition: all 0.2s;
  }

  .file-card-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }
  .file-card-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--border-radius-md);
    flex-shrink: 0;
  }
  .icon-model {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .icon-ortho {
    background: var(--color-info-bg);
    color: var(--color-info);
  }
  .icon-photo {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }
  .icon-work {
    background: rgba(30, 58, 95, 0.08);
    color: var(--color-primary);
  }

  .file-card-info {
    flex: 1;
    min-width: 0;
  }
  .required-mark {
    color: var(--color-error);
  }
  .file-card-count {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-600);
  }
  .btn-sm {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: var(--font-size-xs);
  }

  /* Drop zone (carte vide) */
  .file-drop-zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xl) var(--spacing-md);
    color: var(--color-neutral-500);
    font-size: var(--font-size-xs);
    text-align: center;
    transition: all 0.2s;
  }
  .file-drop-formats {
    font-size: 0.65rem;
    color: var(--color-neutral-500);
    opacity: 0.7;
  }

  .file-list {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: 160px;
    overflow-y: auto;
  }
  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
    color: var(--color-neutral-800);
  }
  .file-remove {
    background: none;
    border: none;
    color: var(--color-neutral-500);
    cursor: pointer;
    padding: 2px;
    border-radius: var(--border-radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s;
    opacity: 0;
  }
  .file-remove:hover {
    background: var(--color-error-bg);
    color: var(--color-error);
  }

  /* ===== RECAP ===== */
  .recap-title {
    margin: 0 0 var(--spacing-xl);
    font-size: var(--font-size-lg);
    font-weight: 700;
    color: var(--color-neutral-900);
    text-align: center;
  }

  .recap-warnings {
    margin-bottom: var(--spacing-xl);
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--color-warning-bg);
    border: 1px solid var(--color-warning-border);
    border-radius: var(--border-radius-md);
  }
  .recap-warnings-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-warning);
  }
  .recap-warnings-list {
    margin: var(--spacing-sm) 0 0;
    padding-left: var(--spacing-xl);
    font-size: var(--font-size-xs);
    color: var(--color-neutral-800);
  }

  .recap-block {
    background: var(--color-neutral-200);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    margin-bottom: var(--spacing-lg);
    overflow: hidden;
  }
  .recap-block:last-child {
    margin-bottom: 0;
  }
  .recap-block-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--color-neutral-300);
    border-bottom: 1px solid var(--color-neutral-400);
  }
  .recap-edit-btn {
    background: none;
    border: none;
    color: var(--color-primary);
    font-size: var(--font-size-xs);
    font-weight: 600;
    cursor: pointer;
    font-family: var(--font-family);
    text-decoration: underline;
    padding: 2px var(--spacing-sm);
  }

  /* ===== FORM ACTIONS ===== */
  .form-actions {
    position: sticky;
    bottom: 0;
    z-index: 5;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md) 0;
    border-top: 1px solid var(--color-neutral-300);
    margin-top: var(--spacing-sm);
    background: var(--color-neutral-200);
  }
  .form-actions-left {
    display: flex;
    gap: var(--spacing-md);
  }
  .btn-lg {
    padding: var(--spacing-md) var(--spacing-2xl);
    font-size: var(--font-size-base);
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* ===== STATUS ===== */
  .status-message {
    margin-bottom: var(--spacing-lg);
  }

  /* ===== LABELS ===== */
  :global(label.meta-label) {
    margin-top: var(--spacing-md);
  }
  .field-col :global(label.meta-label:first-child) {
    margin-top: 0;
  }


  .section-files {
    background: var(--color-neutral-100);
    border-color: var(--color-neutral-300);
  }

  /* ===== RESPONSIVE ===== */
  @media (max-width: 900px) {
    main {
      padding: var(--spacing-xl);
    }
    .field-row-2,
    .recap-grid {
      grid-template-columns: 1fr;
    }
    .op-summary {
      grid-template-columns: 1fr 1fr;
    }

    .stepper {
      flex-direction: column;
      align-items: stretch;
      gap: var(--spacing-sm);
    }
    .step {
      width: 100%;
      justify-content: flex-start;
    }
    .step-text {
      display: flex;
    }
    .step-desc {
      font-size: var(--font-size-xs);
    }
    .step-connector {
      width: 2px;
      height: 16px;
      min-width: 2px;
      margin: 0 0 0 14px;
      align-self: flex-start;
    }
  }

  @media (max-width: 1100px) {
    .file-cards {
      grid-template-columns: repeat(auto-fit, minmax(240px, 300px));
    }
  }
</style>
