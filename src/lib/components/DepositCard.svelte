<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let title = "";
  export let operationCode = "—";
  export let operationSite = "—";
  export let structureType = "—";
  export let hasModel = false;
  export let hasPhotos = false;
  export let hasOrtho = false;
  export let hasWork = false;
  export let active = false;
  export let revisionTagged = false;

  const dispatch = createEventDispatcher();
</script>

<button
  class="deposit-card"
  class:active
  type="button"
  on:click={() => dispatch("click")}
>
  <div class="card-header">
    <div class="title-wrap">
      <span class="card-title">{title}</span>
      <span class="op-badge">{operationCode}</span>
      <span class="op-site">{operationSite}</span>
    </div>
  </div>

  <div class="card-chips">
    <span class="chip">{structureType}</span>
    {#if revisionTagged}
      <span class="chip chip-revision">A modifier</span>
    {/if}
  </div>

  <div class="card-divider"></div>

  <div class="card-footer">
    <span class="file-pill" class:present={hasModel}>Modèle 3D</span>
    <span class="file-pill" class:present={hasPhotos}>Photos</span>
    <span class="file-pill" class:present={hasOrtho}>Ortho</span>
    <span class="file-pill" class:present={hasWork}>Travail</span>
  </div>
</button>

<style>
  .deposit-card {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    background: #fff;
    border: 1px solid rgba(45, 55, 72, 0.08);
    border-radius: var(--border-radius-lg);
    text-align: left;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .deposit-card:hover {
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.08);
    border-color: rgba(30, 58, 95, 0.35);
    transform: translateY(-1px);
  }

  .deposit-card.active {
    border-color: rgba(30, 58, 95, 0.5);
    box-shadow: 0 10px 24px rgba(15, 23, 42, 0.12);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .title-wrap {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .card-title {
    font-size: var(--font-size-base);
    font-weight: 700;
    color: var(--color-neutral-900);
  }

  .op-badge {
    padding: 0 0 0 10px;
    margin-left: 2px;
    border-left: 2px solid rgba(30, 58, 95, 0.25);
    font-size: var(--font-size-base);
    font-weight: 700;
    color: var(--color-primary);
    white-space: nowrap;
    background: none;
  }

  .op-site {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--color-neutral-600);
    white-space: nowrap;
  }

  .card-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    padding: 4px 10px;
    border-radius: 999px;
    background: #f7f9fc;
    border: 1px solid rgba(45, 55, 72, 0.08);
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--color-neutral-700);
  }

  .chip-revision {
    background: rgba(245, 158, 11, 0.12);
    border-color: rgba(245, 158, 11, 0.35);
    color: #92400e;
  }

  .card-divider {
    height: 1px;
    background: rgba(45, 55, 72, 0.08);
    margin-top: 2px;
  }

  .card-footer {
    display: flex;
    gap: var(--spacing-xs);
    flex-wrap: wrap;
  }

  .file-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 5px 10px;
    border-radius: 8px;
    background: #f3f5f8;
    color: var(--color-neutral-500);
    border: 1px solid rgba(45, 55, 72, 0.08);
    font-size: var(--font-size-xs);
    font-weight: 600;
  }

  .file-pill.present {
    background: rgba(47, 133, 90, 0.12);
    color: var(--color-success);
    border-color: rgba(47, 133, 90, 0.25);
  }
</style>
