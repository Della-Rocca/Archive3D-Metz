<script lang="ts">
    import type { OperationMeta, Presets } from "$lib/types/deposit";

    export let operation: OperationMeta;
    export let presets: Presets;
    export let errors: Record<string, string> = {};

    function handleOperationSelect(e: Event) {
        const code = (e.target as HTMLSelectElement).value;
        const found = presets.operations.find((op) => op.code === code);
        if (found) {
            operation = { ...found };
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
            </div>
        </div>
    </div>
</section>

<style>
    /* Styles copiés et adaptés */
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

    @media (max-width: 900px) {
        .op-summary {
            grid-template-columns: repeat(2, minmax(120px, 1fr));
        }
    }
</style>
