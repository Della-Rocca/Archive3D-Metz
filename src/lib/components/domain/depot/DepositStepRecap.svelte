<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { OperationMeta, StructureMeta } from "$lib/types/deposit";

    export let operation: OperationMeta;
    export let structure: StructureMeta;
    export let filesCounts: {
        model: number;
        ortho: number;
        photo: number;
        work: number;
    };
    export let recapIssues: string[] = [];

    const dispatch = createEventDispatcher();

    $: totalFilesCount =
        filesCounts.model +
        filesCounts.ortho +
        filesCounts.photo +
        filesCounts.work;

    function editStep(step: number) {
        dispatch("edit", step);
    }
</script>

<section class="form-section recap-section">
    <div class="section-body">
        <h2 class="recap-title">Vérification avant dépôt</h2>

        <!-- Warnings optionnels -->
        {#if recapIssues.length > 0}
            <div class="recap-warnings">
                <div class="recap-warnings-header">
                    <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
                        ><path
                            d="M12 9V13M12 17H12.01M10.29 3.86L1.82 18A2 2 0 003.54 21H20.46A2 2 0 0022.18 18L13.71 3.86A2 2 0 0010.29 3.86Z"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        /></svg
                    >
                    <span
                        >{recapIssues.length} champ{recapIssues.length > 1
                            ? "s"
                            : ""}
                        optionnel{recapIssues.length > 1 ? "s" : ""} non renseigné{recapIssues.length >
                        1
                            ? "s"
                            : ""}</span
                    >
                </div>
                <ul class="recap-warnings-list">
                    {#each recapIssues as issue}
                        <li>{issue}</li>
                    {/each}
                </ul>
            </div>
        {/if}

        <!-- Bloc Opération -->
        <div class="recap-block">
            <div class="recap-block-header">
                <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
                    ><path
                        d="M22 19C22 19.5304 21.7893 20.0391 21.4142 20.4142C21.0391 20.7893 20.5304 21 20 21H4C3.46957 21 2.96086 20.7893 2.58579 20.4142C2.21071 20.0391 2 19.5304 2 19V5C2 4.46957 2.21071 3.96086 2.58579 3.58579C2.96086 3.21071 3.46957 3 4 3H9L11 6H20C20.5304 6 21.0391 6.21071 21.4142 6.58579C21.7893 6.96086 22 7.46957 22 8V19Z"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    /></svg
                >
                <h3>Opération</h3>
                <button class="recap-edit-btn" on:click={() => editStep(1)}
                    >Modifier</button
                >
            </div>
            <div class="recap-grid">
                <div class="recap-field">
                    <span class="recap-label">Code</span><span
                        class="recap-value">{operation.code}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Site</span><span
                        class="recap-value">{operation.site}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Type</span><span
                        class="recap-value {!operation.op_type
                            ? 'recap-empty'
                            : ''}">{operation.op_type || "Non renseigné"}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Responsable</span><span
                        class="recap-value {!operation.responsable
                            ? 'recap-empty'
                            : ''}"
                        >{operation.responsable || "Non renseigné"}</span
                    >
                </div>
            </div>
        </div>

        <!-- Bloc Structure -->
        <div class="recap-block">
            <div class="recap-block-header">
                <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
                    ><path
                        d="M20 7L12 3L4 7M20 7L12 11M20 7V17L12 21M12 11L4 7M12 11V21M4 7V17L12 21"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    /></svg
                >
                <h3>Structure</h3>
                <button class="recap-edit-btn" on:click={() => editStep(2)}
                    >Modifier</button
                >
            </div>
            <div class="recap-grid">
                <div class="recap-field">
                    <span class="recap-label">Identifiant</span><span
                        class="recap-value">{structure.id}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Type</span><span
                        class="recap-value">{structure.st_type}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Auteur</span><span
                        class="recap-value {!structure.model_author
                            ? 'recap-empty'
                            : ''}"
                        >{structure.model_author || "Non renseigné"}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Déposant</span><span
                        class="recap-value {!structure.depositor
                            ? 'recap-empty'
                            : ''}"
                        >{structure.depositor || "Non renseigné"}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Logiciel</span><span
                        class="recap-value {!structure.software
                            ? 'recap-empty'
                            : ''}">{structure.software || "Non renseigné"}</span
                    >
                </div>
                <div class="recap-field">
                    <span class="recap-label">Polygones</span><span
                        class="recap-value"
                        >{structure.faces_count
                            ? parseInt(structure.faces_count).toLocaleString(
                                  "fr-FR",
                              )
                            : "—"}</span
                    >
                </div>
                {#if structure.description}
                    <div class="recap-field recap-field-full">
                        <span class="recap-label">Description</span><span
                            class="recap-value recap-desc"
                            >{structure.description}</span
                        >
                    </div>
                {/if}
            </div>
        </div>

        <!-- Bloc Fichiers -->
        <div class="recap-block">
            <div class="recap-block-header">
                <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
                    ><path
                        d="M14 2H6C4.89543 2 4 2.89543 4 4V20C4 21.1046 4.89543 22 6 22H18C19.1046 22 20 21.1046 20 20V8L14 2Z"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    /><path
                        d="M14 2V8H20"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    /></svg
                >
                <h3>Fichiers ({totalFilesCount})</h3>
                <button class="recap-edit-btn" on:click={() => editStep(3)}
                    >Modifier</button
                >
            </div>
            <div class="recap-files-summary">
                <div
                    class="recap-file-row"
                    class:recap-file-ok={filesCounts.model > 0}
                >
                    <span class="recap-file-icon"
                        >{filesCounts.model > 0 ? "✓" : "—"}</span
                    >
                    <span>Modèles 3D</span>
                    <span class="recap-file-count">{filesCounts.model}</span>
                </div>
                <div
                    class="recap-file-row"
                    class:recap-file-ok={filesCounts.ortho > 0}
                >
                    <span class="recap-file-icon"
                        >{filesCounts.ortho > 0 ? "✓" : "—"}</span
                    >
                    <span>Orthomosaïques</span>
                    <span class="recap-file-count">{filesCounts.ortho}</span>
                </div>
                <div
                    class="recap-file-row"
                    class:recap-file-ok={filesCounts.photo > 0}
                >
                    <span class="recap-file-icon"
                        >{filesCounts.photo > 0 ? "✓" : "—"}</span
                    >
                    <span>Photos</span>
                    <span class="recap-file-count">{filesCounts.photo}</span>
                </div>
                <div
                    class="recap-file-row"
                    class:recap-file-ok={filesCounts.work > 0}
                >
                    <span class="recap-file-icon"
                        >{filesCounts.work > 0 ? "✓" : "—"}</span
                    >
                    <span>Fichiers de travail</span>
                    <span class="recap-file-count">{filesCounts.work}</span>
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
    .section-body {
        padding: var(--spacing-lg);
    }

    /* ===== RECAPITULATIF ===== */
    .recap-section {
        max-width: 800px;
        margin: 0 auto var(--spacing-xl);
    }
    .recap-title {
        font-size: var(--font-size-lg);
        margin: 0 0 var(--spacing-lg);
        color: var(--color-neutral-900);
        text-align: center;
    }

    .recap-warnings {
        background: var(--color-warning-bg);
        border: 1px solid var(--color-warning);
        border-radius: var(--border-radius-md);
        padding: var(--spacing-md);
        margin-bottom: var(--spacing-lg);
    }
    .recap-warnings-header {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        color: var(--color-warning-dark);
        font-weight: 600;
        margin-bottom: var(--spacing-xs);
    }
    .recap-warnings-list {
        margin: 0;
        padding-left: 24px;
        font-size: var(--font-size-sm);
        color: var(--color-neutral-700);
    }
    .recap-warnings-list li {
        margin-bottom: 2px;
    }

    /* Recap Blocks */
    .recap-block {
        margin-bottom: var(--spacing-xl);
        padding-bottom: var(--spacing-lg);
        border-bottom: 1px solid var(--color-neutral-200);
    }
    .recap-block:last-child {
        border-bottom: none;
        padding-bottom: 0;
        margin-bottom: 0;
    }

    .recap-block-header {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        margin-bottom: var(--spacing-md);
        color: var(--color-neutral-500);
    }
    .recap-block-header h3 {
        margin: 0;
        font-size: var(--font-size-md);
        font-weight: 600;
        color: var(--color-neutral-800);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }
    .recap-edit-btn {
        margin-left: auto;
        font-size: var(--font-size-xs);
        background: none;
        border: 1px solid var(--color-neutral-300);
        padding: 2px 8px;
        border-radius: var(--border-radius-sm);
        cursor: pointer;
        color: var(--color-neutral-600);
        transition: all 0.2s;
    }
    .recap-edit-btn:hover {
        background: var(--color-neutral-100);
        color: var(--color-primary);
        border-color: var(--color-primary);
    }

    .recap-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: var(--spacing-md) var(--spacing-lg);
    }

    .recap-field {
        display: flex;
        flex-direction: column;
    }
    .recap-field-full {
        grid-column: 1 / -1;
    }
    .recap-label {
        font-size: 0.7rem;
        color: var(--color-neutral-600);
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }
    .recap-value {
        font-size: var(--font-size-sm);
        font-weight: 600;
        color: var(--color-neutral-900);
    }
    .recap-empty {
        color: var(--color-neutral-500);
        font-style: italic;
        font-weight: 400;
    }
    .recap-desc {
        font-style: italic;
        font-weight: 400;
    }

    .recap-files-summary {
        padding: var(--spacing-sm) var(--spacing-lg);
    }
    .recap-file-row {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        padding: var(--spacing-xs) 0;
        font-size: var(--font-size-sm);
        color: var(--color-neutral-600);
    }
    .recap-file-row.recap-file-ok {
        color: var(--color-neutral-900);
    }
    .recap-file-icon {
        width: 18px;
        text-align: center;
        font-weight: 700;
    }
    .recap-file-ok .recap-file-icon {
        color: var(--color-success);
    }
    .recap-file-count {
        margin-left: auto;
        font-weight: 600;
        font-size: var(--font-size-xs);
        color: var(--color-neutral-700);
    }
</style>
