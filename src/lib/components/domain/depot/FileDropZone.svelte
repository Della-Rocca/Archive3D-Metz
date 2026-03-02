<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { filename } from "$lib/utils/file";

    export let title: string;
    export let required = false;
    export let files: string[] = [];
    export let dropId: string;
    export let dragOver: string | null = null;
    export let error = "";

    const dispatch = createEventDispatcher();

    $: isActive = dragOver === dropId;

    function handleRemove(index: number) {
        if (index >= 0 && index < files.length) {
            dispatch("remove", index);
        }
    }
</script>

<div
    class="file-card"
    class:filled={files.length > 0}
    class:file-card-dragover={isActive}
    class:input-error={!!error}
    data-drop={dropId}
>
    <div class="file-card-header">
        <div class="title-wrap">
            <h3>
                {title}
                {#if required}<span class="required">*</span>{/if}
            </h3>
            <span class="file-count"
                >{files.length} fichier{files.length > 1 ? "s" : ""}</span
            >
        </div>
    </div>

    {#if error}
        <p
            class="field-error-text"
            style="margin-top:-8px; margin-bottom:12px; font-size:0.8rem; color:var(--color-error);"
        >
            {error}
        </p>
    {/if}

    <div class="file-container">
        {#if files.length === 0}
            <div class="empty-placeholder">
                <div class="drop-callout">
                    <span class="drop-icon" aria-hidden="true">⬇</span>
                    <span class="empty-text">Glisser-déposer vos fichiers</span>
                    <span class="empty-subtext">Déposez les fichiers dans cette zone</span>
                </div>
                <div class="empty-separator">
                    <span>ou</span>
                </div>
                <button
                    type="button"
                    class="empty-add-btn"
                    on:click={() => dispatch("add")}
                >
                    Choisir des fichiers
                </button>
            </div>
        {:else}
            <div class="filled-content">
                <div class="file-list-scroll">
                    <ul class="file-list">
                        {#each files as file, i}
                            <li class="file-item">
                                <span class="file-name" title={file}
                                    >{filename(file)}</span
                                >
                                <button
                                    class="remove-file-btn"
                                    on:click={() => handleRemove(i)}>×</button
                                >
                            </li>
                        {/each}
                    </ul>
                </div>
                <div class="filled-footer">
                    <!-- Contenu additionnel (ex: polygones) -->
                    <slot />
                    <div class="file-drop-hint">
                        Glissez des fichiers pour en ajouter
                    </div>
                    <div class="file-action-row">
                        <button
                            type="button"
                            class="empty-add-btn compact"
                            on:click={() => dispatch("add")}
                        >
                            Choisir des fichiers
                        </button>
                    </div>
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .file-card {
        background: #fcfdff;
        border: 1px solid rgba(30, 58, 95, 0.14);
        border-radius: var(--border-radius-lg);
        padding: var(--spacing-md);
        transition: all 0.2s ease;
        display: flex;
        flex-direction: column;
        height: 320px;
        min-height: 320px;
        position: relative;
        overflow: hidden;
        box-shadow:
            0 1px 2px rgba(15, 23, 42, 0.05),
            0 6px 16px rgba(15, 23, 42, 0.06);
    }

    .file-card:hover {
        border-color: rgba(30, 58, 95, 0.28);
        box-shadow:
            0 2px 4px rgba(15, 23, 42, 0.08),
            0 10px 24px rgba(15, 23, 42, 0.08);
        transform: translateY(-1px);
    }

    /* État rempli */
    .file-card.filled {
        border-color: rgba(30, 58, 95, 0.32);
        background: linear-gradient(
            to bottom,
            rgba(252, 253, 255, 1),
            rgba(246, 249, 253, 1)
        );
    }

    /* État erreur */
    .file-card.input-error {
        border-color: var(--color-error);
        background: #fff5f5;
    }

    /* DRAG OVER state */
    .file-card-dragover {
        border: 2px dashed var(--color-primary) !important;
        background: #eff6ff !important;
        transform: scale(1.01);
        box-shadow: 0 10px 30px -10px rgba(30, 58, 95, 0.25);
        z-index: 10;
    }
    .file-card-dragover::after {
        content: "";
        position: absolute;
        inset: 0;
        background: rgba(30, 58, 95, 0.03);
        pointer-events: none;
        animation: dropPulse 1.5s infinite;
    }

    @keyframes dropPulse {
        0% {
            opacity: 0;
        }
        50% {
            opacity: 1;
        }
        100% {
            opacity: 0;
        }
    }

    /* Header */
    .file-card-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: var(--spacing-sm);
        gap: var(--spacing-sm);
    }
    .title-wrap {
        min-width: 0;
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .file-card-header h3 {
        margin: 0;
        font-size: var(--font-size-sm);
        color: var(--color-neutral-800);
        font-weight: 700;
    }
    .file-count {
        font-size: var(--font-size-xs);
        color: var(--color-neutral-600);
    }
    .required {
        color: var(--color-error);
        margin-left: 2px;
    }

    /* Conteneur liste/empty */
    .file-container {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
    }

    /* Empty state */
    .empty-placeholder {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 8px;
        color: var(--color-neutral-500);
        border: 2px dashed var(--color-neutral-300);
        border-radius: var(--border-radius-md);
        margin-top: var(--spacing-xs);
        padding: var(--spacing-md);
        background: linear-gradient(
            to bottom,
            rgba(30, 58, 95, 0.02),
            rgba(30, 58, 95, 0.04)
        );
    }
    .drop-callout {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        text-align: center;
    }
    .drop-icon {
        width: 30px;
        height: 30px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border-radius: 50%;
        background: rgba(30, 58, 95, 0.12);
        color: var(--color-primary);
        font-size: 1rem;
        font-weight: 700;
    }
    .empty-text {
        font-size: var(--font-size-xs);
        font-weight: 700;
        color: var(--color-neutral-700);
    }
    .empty-subtext {
        font-size: 0.68rem;
        color: var(--color-neutral-500);
    }
    .empty-separator {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        color: var(--color-neutral-500);
        font-size: 0.68rem;
        text-transform: uppercase;
        letter-spacing: 0.08em;
    }
    .empty-separator::before,
    .empty-separator::after {
        content: "";
        height: 1px;
        flex: 1;
        background: var(--color-neutral-300);
    }
    .empty-add-btn {
        width: 100%;
        max-width: 220px;
        height: 34px;
        border-radius: 999px;
        border: 1px solid var(--color-primary);
        background: #fff;
        color: var(--color-primary);
        font-size: var(--font-size-xs);
        font-weight: 700;
        cursor: pointer;
        transition: all 0.18s ease;
    }
    .empty-add-btn:hover {
        background: var(--color-primary);
        color: #fff;
    }
    .empty-add-btn.compact {
        max-width: 180px;
        height: 30px;
        font-size: 0.72rem;
    }

    .filled-content {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .file-list-scroll {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }

    /* Liste fichiers */
    .file-list {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .file-item {
        background: #fff;
        border: 1px solid var(--color-neutral-300);
        border-radius: 6px;
        padding: 6px 8px;
        font-size: var(--font-size-xs);
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .file-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        margin-right: 8px;
        font-weight: 600;
        color: var(--color-neutral-700);
    }

    .remove-file-btn {
        background: none;
        border: none;
        color: var(--color-neutral-400);
        cursor: pointer;
        font-size: 1rem;
        line-height: 1;
        padding: 0 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        height: 100%;
    }
    .remove-file-btn:hover {
        color: var(--color-error);
        background: rgba(220, 38, 38, 0.1);
    }

    .file-drop-hint {
        margin-top: 2px;
        text-align: center;
        font-size: 0.68rem;
        color: var(--color-neutral-500);
        font-style: italic;
        background: rgba(0, 0, 0, 0.02);
        border-radius: 4px;
        padding: 3px 4px;
    }
    .file-action-row {
        display: flex;
        justify-content: center;
        padding-top: 2px;
    }

    @media (max-width: 900px) {
        .file-card {
            height: 280px;
            min-height: 280px;
        }
        .file-list {
            gap: 5px;
        }
    }
</style>
