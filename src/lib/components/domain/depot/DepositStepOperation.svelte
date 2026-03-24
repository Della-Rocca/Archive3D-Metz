<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { OperationMeta, Presets } from "$lib/types/deposit";

    export let operation: OperationMeta;
    export let presets: Presets;
    export let errors: Record<string, string> = {};
    export let canAdmin = false;

    const dispatch = createEventDispatcher<{
        operationCreated: OperationMeta;
    }>();

    // --- Sélection depuis les presets ---
    function handleOperationSelect(e: Event) {
        const code = (e.target as HTMLSelectElement).value;
        const found = presets.operations.find((op) => op.code === code);
        if (found) {
            operation = { ...found };
        }
    }

    // --- Création rapide ---
    let showQuickCreate = false;
    let quickSaving = false;
    let quickError = "";
    let quickSuccess = false;

    let quickForm: OperationMeta = {
        code: "",
        site: "",
        op_type: "",
        responsable: "",
    };

    function openQuickCreate() {
        quickForm = { code: "", site: "", op_type: "", responsable: "" };
        quickError = "";
        quickSuccess = false;
        showQuickCreate = true;
    }

    function cancelQuickCreate() {
        showQuickCreate = false;
        quickError = "";
        quickSuccess = false;
    }

    // Validation locale du formulaire rapide
    $: quickFormValid =
        quickForm.code.trim().length > 0 &&
        quickForm.site.trim().length > 0 &&
        !FORBIDDEN_QUICK.test(quickForm.code) &&
        !FORBIDDEN_QUICK.test(quickForm.site);

    // Mêmes caractères interdits que safe_segment Rust
    const FORBIDDEN_QUICK = /[\/\\\0:\.\.]/;

    async function handleQuickCreate() {
        if (!quickFormValid) return;

        // Vérification doublons code
        const alreadyExists = presets.operations.some(
            (op) => op.code.toLowerCase() === quickForm.code.trim().toLowerCase(),
        );
        if (alreadyExists) {
            quickError = `Le code opération "${quickForm.code.trim()}" existe déjà.`;
            return;
        }

        quickSaving = true;
        quickError = "";
        try {
            // Construit les presets mis à jour avec la nouvelle opération
            const newOp: OperationMeta = {
                code: quickForm.code.trim(),
                site: quickForm.site.trim(),
                op_type: quickForm.op_type.trim(),
                responsable: quickForm.responsable.trim(),
            };

            const updatedPresets: typeof presets = {
                ...presets,
                operations: [...presets.operations, newOp],
                sites: presets.sites.includes(newOp.site)
                    ? presets.sites
                    : [...presets.sites, newOp.site],
                operation_types:
                    newOp.op_type && !presets.operation_types.includes(newOp.op_type)
                        ? [...presets.operation_types, newOp.op_type]
                        : presets.operation_types,
                responsables:
                    newOp.responsable &&
                    !presets.responsables.includes(newOp.responsable)
                        ? [...presets.responsables, newOp.responsable]
                        : presets.responsables,
            };

            // Import dynamique pour ne pas alourdir le bundle si non-admin
            const { updateMetadataPresets } = await import("$lib/api/deposit");
            await updateMetadataPresets(updatedPresets);

            quickSuccess = true;
            // Sélectionne automatiquement la nouvelle opération
            operation = { ...newOp };

            // Notifie le parent pour recharger les presets
            dispatch("operationCreated", newOp);

            setTimeout(() => {
                showQuickCreate = false;
                quickSuccess = false;
            }, 1200);
        } catch (e) {
            quickError = `Erreur lors de la création : ${e}`;
        } finally {
            quickSaving = false;
        }
    }
</script>

<section class="form-section">
    <div class="section-header">
        <h2>1. Identification de l'opération</h2>
    </div>
    <div class="form-body">
        <div class="form-row">
            <div class="field-col">
                <label class="field-label required" for="op-select">
                    Sélection de l'opération
                </label>
                <div class="meta-select-wrapper">
                    <select
                        id="op-select"
                        class="meta-select"
                        class:input-error={!!errors["operation.code"]}
                        on:change={handleOperationSelect}
                    >
                        <option value=""
                            >-- Choisir une opération existante --</option
                        >
                        {#each presets.operations as op}
                            <option value={op.code}
                                >{op.code} - {op.site}</option
                            >
                        {/each}
                    </select>
                </div>
                {#if errors["operation.code"]}
                    <span class="field-error">{errors["operation.code"]}</span>
                {/if}

                <div
                    class="op-summary"
                    class:op-summary-empty={!operation.code}
                >
                    <div class="op-summary-item">
                        <span class="op-summary-label">Code</span>
                        <span
                            class="op-summary-value"
                            class:op-summary-placeholder={!operation.code}
                            >{operation.code || "---"}</span
                        >
                    </div>
                    <div class="op-summary-item">
                        <span class="op-summary-label">Site</span>
                        <span
                            class="op-summary-value"
                            class:op-summary-placeholder={!operation.site}
                            >{operation.site || "---"}</span
                        >
                    </div>
                    <div class="op-summary-item">
                        <span class="op-summary-label">Type</span>
                        <span
                            class="op-summary-value"
                            class:op-summary-placeholder={!operation.op_type}
                            >{operation.op_type || "---"}</span
                        >
                    </div>
                    <div class="op-summary-item">
                        <span class="op-summary-label">Responsable</span>
                        <span
                            class="op-summary-value"
                            class:op-summary-placeholder={!operation.responsable}
                            >{operation.responsable || "---"}</span
                        >
                    </div>
                </div>

                <!-- Bouton création rapide (admin seulement) -->
                {#if canAdmin}
                    {#if !showQuickCreate}
                        <button
                            type="button"
                            class="btn btn-outline"
                            style="margin-top: var(--spacing-md); align-self: flex-start; gap: 6px;"
                            on:click={openQuickCreate}
                        >
                            <svg viewBox="0 0 24 24" fill="none" width="14" height="14" aria-hidden="true">
                                <path d="M12 5V19M5 12H19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                            </svg>
                            Créer une nouvelle opération
                        </button>
                    {:else}
                        <!-- Formulaire création rapide -->
                        <div class="quick-create-panel">
                            <div class="quick-create-header">
                                <span class="quick-create-title">Nouvelle opération</span>
                                <button
                                    type="button"
                                    class="quick-create-close"
                                    on:click={cancelQuickCreate}
                                    aria-label="Fermer"
                                >
                                    <svg viewBox="0 0 24 24" fill="none" width="14" height="14">
                                        <path d="M18 6L6 18M6 6L18 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                                    </svg>
                                </button>
                            </div>

                            <div class="quick-create-grid">
                                <div class="quick-field">
                                    <label class="quick-label required" for="qc-code">Code opération</label>
                                    <input
                                        id="qc-code"
                                        class="quick-input"
                                        class:quick-input-error={quickForm.code.trim() && FORBIDDEN_QUICK.test(quickForm.code)}
                                        bind:value={quickForm.code}
                                        placeholder="ex: 202501"
                                        disabled={quickSaving}
                                    />
                                    {#if quickForm.code.trim() && FORBIDDEN_QUICK.test(quickForm.code)}
                                        <span class="quick-field-error">Caractères interdits ( / \ : .. )</span>
                                    {/if}
                                </div>
                                <div class="quick-field">
                                    <label class="quick-label required" for="qc-site">Site</label>
                                    <input
                                        id="qc-site"
                                        class="quick-input"
                                        class:quick-input-error={quickForm.site.trim() && FORBIDDEN_QUICK.test(quickForm.site)}
                                        bind:value={quickForm.site}
                                        placeholder="ex: Metz_Centre"
                                        disabled={quickSaving}
                                    />
                                    {#if quickForm.site.trim() && FORBIDDEN_QUICK.test(quickForm.site)}
                                        <span class="quick-field-error">Caractères interdits ( / \ : .. )</span>
                                    {/if}
                                </div>
                                <div class="quick-field">
                                    <label class="quick-label" for="qc-type">Type d'opération</label>
                                    <input
                                        id="qc-type"
                                        class="quick-input"
                                        bind:value={quickForm.op_type}
                                        list="qc-type-list"
                                        placeholder="ex: Fouille préventive"
                                        disabled={quickSaving}
                                    />
                                    <datalist id="qc-type-list">
                                        {#each presets.operation_types as t}
                                            <option value={t}></option>
                                        {/each}
                                    </datalist>
                                </div>
                                <div class="quick-field">
                                    <label class="quick-label" for="qc-responsable">Responsable</label>
                                    <input
                                        id="qc-responsable"
                                        class="quick-input"
                                        bind:value={quickForm.responsable}
                                        list="qc-resp-list"
                                        placeholder="ex: Dupont Marie"
                                        disabled={quickSaving}
                                    />
                                    <datalist id="qc-resp-list">
                                        {#each presets.responsables as r}
                                            <option value={r}></option>
                                        {/each}
                                    </datalist>
                                </div>
                            </div>

                            {#if quickError}
                                <p class="quick-error">{quickError}</p>
                            {/if}
                            {#if quickSuccess}
                                <p class="quick-success">Opération créée et sélectionnée.</p>
                            {/if}

                            <div class="quick-create-actions">
                                <button
                                    type="button"
                                    class="btn btn-secondary"
                                    on:click={cancelQuickCreate}
                                    disabled={quickSaving}
                                >
                                    Annuler
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-primary"
                                    on:click={handleQuickCreate}
                                    disabled={!quickFormValid || quickSaving}
                                >
                                    {#if quickSaving}
                                        <span class="spinner"></span> Enregistrement...
                                    {:else}
                                        Créer l'opération
                                    {/if}
                                </button>
                            </div>
                        </div>
                    {/if}
                {/if}
            </div>
        </div>
    </div>
</section>

<style>
    .form-section {
        background: #fff;
        border: 1px solid var(--color-neutral-300);
        border-radius: var(--border-radius-lg);
        overflow: hidden;
        margin-bottom: var(--spacing-xl);
    }
    .section-header {
        background: var(--color-neutral-100);
        padding: var(--spacing-md) var(--spacing-lg);
        border-bottom: 1px solid var(--color-neutral-300);
    }
    .section-header h2 {
        margin: 0;
        font-size: var(--font-size-lg);
        color: var(--color-neutral-800);
    }

    .form-body {
        padding: var(--spacing-lg);
    }

    .form-row {
        display: grid;
        grid-template-columns: 1fr;
        gap: var(--spacing-lg);
        align-items: start;
    }

    .field-col {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }

    .field-label {
        font-size: var(--font-size-sm);
        font-weight: 600;
        color: var(--color-neutral-700);
    }
    .field-label.required::after {
        content: "*";
        color: var(--color-error);
        margin-left: 2px;
    }

    .field-error {
        font-size: var(--font-size-xs);
        color: var(--color-error);
    }

    /* Op Summary Box */
    .op-summary {
        margin-top: var(--spacing-md);
        padding: var(--spacing-md) var(--spacing-lg);
        background: #f7f9fc;
        border-radius: var(--border-radius-md);
        display: grid;
        grid-template-columns: repeat(4, minmax(120px, 1fr));
        gap: var(--spacing-md);
        border: 1px solid rgba(45, 55, 72, 0.08);
        align-items: center;
    }
    .op-summary-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
        font-size: var(--font-size-sm);
    }
    .op-summary-label {
        color: var(--color-neutral-500);
        font-size: var(--font-size-xs);
        text-transform: uppercase;
        letter-spacing: 0.04em;
        font-weight: 700;
    }
    .op-summary-value {
        font-weight: 700;
        color: var(--color-neutral-900);
    }

    .op-summary-empty {
        border-style: dashed;
        background: #fcfcfd;
    }
    .op-summary-placeholder {
        color: var(--color-neutral-500);
        font-weight: 500;
        font-style: italic;
    }



    /* --- Panneau création rapide --- */
    .quick-create-panel {
        margin-top: var(--spacing-md);
        padding: var(--spacing-lg);
        background: #f7f9fc;
        border: 1px solid var(--color-neutral-300);
        border-radius: var(--border-radius-md);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
    }

    .quick-create-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    .quick-create-title {
        font-size: var(--font-size-sm);
        font-weight: 700;
        color: var(--color-neutral-800);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }
    .quick-create-close {
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-neutral-500);
        padding: 4px;
        border-radius: var(--border-radius-sm);
        transition: color 0.15s, background 0.15s;
    }
    .quick-create-close:hover {
        color: var(--color-neutral-800);
        background: var(--color-neutral-200);
    }

    .quick-create-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
    }

    .quick-field {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .quick-label {
        font-size: var(--font-size-xs);
        font-weight: 600;
        color: var(--color-neutral-600);
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }
    .quick-label.required::after {
        content: "*";
        color: var(--color-error);
        margin-left: 2px;
    }
    .quick-input {
        padding: 7px 10px;
        border: 1px solid var(--color-neutral-300);
        border-radius: var(--border-radius-sm);
        font-size: var(--font-size-sm);
        background: #fff;
        color: var(--color-neutral-900);
        outline: none;
        transition: border-color 0.15s;
    }
    .quick-input:focus {
        border-color: var(--color-primary);
    }
    .quick-input-error {
        border-color: var(--color-error);
    }
    .quick-field-error {
        font-size: var(--font-size-xs);
        color: var(--color-error);
    }

    .quick-error {
        font-size: var(--font-size-sm);
        color: var(--color-error);
        margin: 0;
        padding: 8px 12px;
        background: color-mix(in srgb, var(--color-error) 8%, transparent);
        border-radius: var(--border-radius-sm);
        border: 1px solid color-mix(in srgb, var(--color-error) 20%, transparent);
    }
    .quick-success {
        font-size: var(--font-size-sm);
        color: var(--color-success, #2d8a4e);
        margin: 0;
        padding: 8px 12px;
        background: color-mix(in srgb, var(--color-success, #2d8a4e) 10%, transparent);
        border-radius: var(--border-radius-sm);
        border: 1px solid color-mix(in srgb, var(--color-success, #2d8a4e) 20%, transparent);
        font-weight: 600;
    }

    .quick-create-actions {
        display: flex;
        gap: var(--spacing-sm);
        justify-content: flex-end;
    }

    .spinner {
        display: inline-block;
        width: 12px;
        height: 12px;
        border: 2px solid rgba(255, 255, 255, 0.4);
        border-top-color: #fff;
        border-radius: 50%;
        animation: spin 0.7s linear infinite;
        vertical-align: middle;
    }
    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    @media (max-width: 900px) {
        .op-summary {
            grid-template-columns: repeat(2, minmax(120px, 1fr));
        }
        .quick-create-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
