<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import ComboInput from "$lib/components/ComboInput.svelte";
    import type { StructureMeta, Presets } from "$lib/types/deposit";

    export let structure: StructureMeta;
    export let presets: Presets;
    export let errors: Record<string, string> = {};
    export let countingPolygons = false;
    // selectedSoftware peut être interne, on l'initialise depuis structure.software
    let selectedSoftware: string[] = [];

    // Initialisation réactive : si structure.software change (ex: reset), on met à jour selectedSoftware
    $: {
        if (structure.software) {
            const split = structure.software.split(", ").filter((s) => s);
            // Si différent, on update (pour éviter boucle si update vient de toggle)
            const current = selectedSoftware.join(", ");
            if (current !== structure.software) {
                selectedSoftware = split;
            }
        } else {
            selectedSoftware = [];
        }
    }

    const dispatch = createEventDispatcher();

    function handleTouch(field: string) {
        dispatch("touch", field);
    }

    function toggleSoftware(sw: string) {
        if (selectedSoftware.includes(sw)) {
            selectedSoftware = selectedSoftware.filter((s) => s !== sw);
        } else {
            selectedSoftware = [...selectedSoftware, sw];
        }
        structure.software = selectedSoftware.join(", ");
        handleTouch("structure.software");
    }
</script>

<section class="form-section">
    <div class="section-header">
        <h2>2. Métadonnées de la structure</h2>
    </div>
    <div class="section-body">
        <!-- Identification -->
        <div class="field-group">
            <h3 class="field-group-title">Identification</h3>
            <div class="field-row-2">
                <div class="field-col">
                    <label class="meta-label">Identifiant structure *</label>
                    <input
                        class="meta-input"
                        class:input-error={!!errors["structure.id"]}
                        bind:value={structure.id}
                        on:blur={() => handleTouch("structure.id")}
                        placeholder="ex: ST443"
                    />
                    {#if errors["structure.id"]}
                        <span class="field-error">{errors["structure.id"]}</span
                        >
                    {/if}
                </div>
                <div class="field-col">
                    <label class="meta-label">Type de structure *</label>
                    <div class:combo-error={!!errors["structure.st_type"]}>
                        <ComboInput
                            bind:value={structure.st_type}
                            options={presets.structure_types}
                            placeholder="ex: sépulture"
                            on:change={() => handleTouch("structure.st_type")}
                        />
                    </div>
                    {#if errors["structure.st_type"]}
                        <span class="field-error"
                            >{errors["structure.st_type"]}</span
                        >
                    {/if}
                </div>
            </div>
            <label class="meta-label desc-label">Description</label>
            <textarea
                class="meta-input"
                bind:value={structure.description}
                placeholder="Description libre (optionnel)"
                rows="2"
            ></textarea>
        </div>

        <div class="form-separator"></div>

        <!-- Modèle 3D -->
        <div class="field-group">
            <h3 class="field-group-title">Modèle 3D</h3>
            <div class="field-row-2">
                <div class="field-col">
                    <label class="meta-label">Auteur du modèle</label>
                    <ComboInput
                        bind:value={structure.model_author}
                        options={presets.model_authors}
                        placeholder="ex: Martin Pierre"
                    />
                </div>
                <div class="field-col">
                    <label class="meta-label">Déposant</label>
                    <ComboInput
                        bind:value={structure.depositor}
                        options={presets.depositors}
                        placeholder="ex: Votre nom"
                    />
                </div>
            </div>

            <!-- Logiciels -->
            <div class="software-field">
                <label class="meta-label">Logiciels utilisés</label>
                {#if presets.software_types.length > 0}
                    <div class="checkbox-group">
                        {#each presets.software_types as sw}
                            <button
                                type="button"
                                class="checkbox-chip"
                                class:active={selectedSoftware.includes(sw)}
                                on:click={() => toggleSoftware(sw)}
                            >
                                {sw}
                            </button>
                        {/each}
                    </div>
                {:else}
                    <p class="hint-text">Aucun logiciel configuré.</p>
                {/if}
            </div>

            <div class="field-row-2">
                <div class="field-col">
                    <label class="meta-label">Nombre de photos</label>
                    <input
                        class="meta-input"
                        value={structure.photos_count}
                        readonly
                        placeholder="Calculé automatiquement"
                        disabled
                        style="background: var(--color-neutral-100); color: var(--color-neutral-500);"
                    />
                </div>
                <div class="field-col">
                    <label class="meta-label">Nombre de polygones</label>
                    <input
                        class="meta-input"
                        value={structure.faces_count}
                        readonly
                        disabled
                        style="background: var(--color-neutral-100); color: var(--color-neutral-500);"
                        placeholder={countingPolygons
                            ? "Comptage en cours..."
                            : "Calculé automatiquement"}
                    />
                </div>
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

    .section-body {
        padding: var(--spacing-lg);
    }

    .field-group {
        margin-bottom: var(--spacing-lg);
    }
    .field-group-title {
        font-size: var(--font-size-md);
        color: var(--color-neutral-900);
        margin: 0 0 var(--spacing-md);
        font-weight: 600;
    }

    .field-row-2 {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-lg);
        margin-bottom: var(--spacing-md);
    }

    .field-col {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .meta-label {
        font-size: var(--font-size-sm);
        font-weight: 600;
        color: var(--color-neutral-700);
    }
    .desc-label {
        margin-top: var(--spacing-sm);
        display: block;
        margin-bottom: 6px;
    }

    .field-error {
        font-size: var(--font-size-xs);
        color: var(--color-error);
    }

    .form-separator {
        height: 1px;
        background: var(--color-neutral-200);
        margin: var(--spacing-xl) 0;
    }

    .software-field {
        margin: var(--spacing-lg) 0;
    }
    .checkbox-group {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 8px;
    }
    .checkbox-chip {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 7px 14px;
        background: var(--color-neutral-200);
        border: 1px solid var(--color-neutral-400);
        border-radius: 999px;
        font-size: var(--font-size-sm);
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        color: var(--color-neutral-700);
    }
    .checkbox-chip:hover {
        background: var(--color-neutral-300);
        border-color: var(--color-neutral-500);
    }
    .checkbox-chip.active {
        background: var(--color-primary);
        border-color: var(--color-primary);
        color: var(--color-neutral-100);
        font-weight: 600;
    }
    .hint-text {
        font-size: var(--font-size-xs);
        color: var(--color-neutral-500);
        font-style: italic;
    }
</style>
