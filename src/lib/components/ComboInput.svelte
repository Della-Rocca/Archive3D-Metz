<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let value = "";
    export let options: string[] = [];
    export let placeholder = "";
    export let disabled = false;

    const dispatch = createEventDispatcher();

    let open = false;
    let inputEl: HTMLInputElement;
    let containerEl: HTMLDivElement;

    $: filtered = options;

    function select(opt: string) {
        value = opt;
        dispatch("change", value);
    }

    function handleInput() {
        open = true;
        dispatch("input", value);
    }

    function handleClick() {
        if (disabled) return;
        open = !open;
    }

    function handleBlur(e: FocusEvent) {
        const related = e.relatedTarget as HTMLElement | null;
        if (related && containerEl?.contains(related)) return;
        open = false;
        dispatch("change", value);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            open = false;
            inputEl?.blur();
        }
    }

    function handleChevronClick() {
        if (disabled) return;
        if (open) {
            open = false;
        } else {
            open = true;
            inputEl?.focus();
        }
    }
</script>

<div class="combo" class:disabled bind:this={containerEl}>
    <input
        bind:this={inputEl}
        bind:value
        class="combo-input"
        {placeholder}
        {disabled}
        on:input={handleInput}
        on:click={handleClick}
        on:blur={handleBlur}
        on:keydown={handleKeydown}
        autocomplete="off"
    />
    <button
        type="button"
        class="combo-toggle"
        on:mousedown|preventDefault={handleChevronClick}
        {disabled}
        tabindex="-1"
        aria-label="Ouvrir la liste"
    >
        <svg
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            class:combo-chevron-open={open}
        >
            <path
                d="M6 9l6 6 6-6"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
        </svg>
    </button>
    {#if open && filtered.length > 0}
        <ul class="combo-list">
            {#each filtered as opt}
                <button
                    type="button"
                    class="combo-option"
                    on:mousedown|preventDefault={() => select(opt)}
                >
                    {opt}
                </button>
            {/each}
        </ul>
    {/if}
</div>

<style>
    .combo-toggle svg {
        transition: transform 0.2s ease;
    }

    .combo-chevron-open {
        transform: rotate(180deg);
    }
</style>
