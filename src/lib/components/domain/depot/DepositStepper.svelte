<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let steps: { id: number; label: string; desc: string }[] = [];
    export let currentStep = 1;
    export let step1Valid = false;
    export let step2Valid = false;
    export let step3Valid = false;

    const dispatch = createEventDispatcher();

    function stepStatus(
        stepId: number,
    ): "completed" | "current" | "upcoming" | "error" {
        if (stepId === currentStep) return "current";
        if (stepId < currentStep) {
            if (stepId === 1 && !step1Valid) return "error";
            if (stepId === 2 && !step2Valid) return "error";
            if (stepId === 3 && !step3Valid) return "error";
            return "completed";
        }
        return "upcoming";
    }

    function handleStepClick(stepId: number) {
        if (stepId > currentStep + 1) return;
        dispatch("stepClick", stepId);
    }
</script>

<div class="stepper">
    {#each steps as step, i}
        <button
            class="step"
            class:step-current={stepStatus(step.id) === "current"}
            class:step-completed={stepStatus(step.id) === "completed"}
            class:step-error={stepStatus(step.id) === "error"}
            class:step-upcoming={stepStatus(step.id) === "upcoming"}
            on:click={() => handleStepClick(step.id)}
            disabled={step.id > currentStep + 1}
        >
            <span class="step-circle">
                {#if stepStatus(step.id) === "completed"}
                    <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
                        ><path
                            d="M20 6L9 17L4 12"
                            stroke="currentColor"
                            stroke-width="3"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        /></svg
                    >
                {:else if stepStatus(step.id) === "error"}
                    !
                {:else}
                    {step.id}
                {/if}
            </span>
            <span class="step-text">
                <span class="step-label">{step.label}</span>
                <span class="step-desc">{step.desc}</span>
            </span>
        </button>
        {#if i < steps.length - 1}
            <div
                class="step-connector"
                class:step-connector-done={step.id < currentStep}
            ></div>
        {/if}
    {/each}
</div>

<style>
    .stepper {
        display: flex;
        align-items: center;
        gap: 0;
        margin-bottom: var(--spacing-2xl);
        padding: var(--spacing-lg) var(--spacing-xl);
        background: var(--color-neutral-100);
        border: 1px solid var(--color-neutral-300);
        border-radius: var(--border-radius-lg);
    }
    .step {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        background: none;
        border: none;
        cursor: pointer;
        padding: var(--spacing-sm) var(--spacing-md);
        border-radius: var(--border-radius-md);
        transition: all 0.15s;
        font-family: var(--font-family);
        flex-shrink: 0;
    }
    .step:disabled {
        cursor: default;
        opacity: 0.5;
    }
    .step:not(:disabled):hover {
        background: var(--color-neutral-200);
    }

    .step-circle {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        border-radius: 50%;
        flex-shrink: 0;
        font-size: var(--font-size-xs);
        font-weight: 700;
        transition: all 0.2s;
    }
    .step-upcoming .step-circle {
        background: var(--color-neutral-300);
        color: var(--color-neutral-600);
    }
    .step-current .step-circle {
        background: var(--color-primary);
        color: var(--color-neutral-100);
        box-shadow: 0 0 0 3px rgba(30, 58, 95, 0.15);
    }
    .step-completed .step-circle {
        background: var(--color-success);
        color: var(--color-neutral-100);
    }
    .step-error .step-circle {
        background: var(--color-error);
        color: var(--color-neutral-100);
    }

    .step-text {
        display: flex;
        flex-direction: column;
        text-align: left;
    }
    .step-label {
        font-size: var(--font-size-xs);
        font-weight: 600;
        color: var(--color-neutral-900);
        line-height: 1.2;
    }
    .step-current .step-label {
        color: var(--color-primary);
    }
    .step-desc {
        font-size: 0.65rem;
        color: var(--color-neutral-600);
        line-height: 1.2;
    }

    .step-connector {
        flex: 1;
        height: 2px;
        min-width: 16px;
        background: var(--color-neutral-400);
        margin: 0 var(--spacing-xs);
        transition: background 0.3s;
    }
    .step-connector-done {
        background: var(--color-success);
    }
</style>
