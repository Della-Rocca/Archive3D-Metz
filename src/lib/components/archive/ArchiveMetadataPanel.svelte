<script lang="ts">
  interface MetadataDisplayItem {
    label: string;
    value: string;
  }

  export let operationItems: MetadataDisplayItem[] = [];
  export let structureItems: MetadataDisplayItem[] = [];
  export let description = "—";

  function isMissing(value: string): boolean {
    return value.trim() === "—";
  }

  function isTechnical(label: string, value: string): boolean {
    const l = label.toLowerCase();
    if (l.includes("code") || l.includes("id") || l.includes("identifiant")) return true;
    if (l.includes("nombre") || l.includes("photo") || l.includes("polygone")) return true;
    if (/[0-9]/.test(value)) return true;
    if (/^[A-Z0-9._-]{4,}$/.test(value.trim())) return true;
    return false;
  }
</script>

<section class="metadata-tech">
  <section class="meta-section">
    <h4 class="section-title">Opération</h4>
    <div class="operation-columns">
      {#each operationItems as item}
        <div class="field" title={item.value}>
          <span class="meta-label">{item.label}</span>
          <span
            class="meta-value"
            class:meta-value-tech={isTechnical(item.label, item.value)}
            class:meta-value-missing={isMissing(item.value)}
            >{item.value}</span
          >
        </div>
      {/each}
    </div>
  </section>

  <section class="meta-section">
    <h4 class="section-title">Structure</h4>
    <div class="structure-columns">
      {#each structureItems as item}
        <div class="field" title={item.value}>
          <span class="meta-label">{item.label}</span>
          <span
            class="meta-value"
            class:meta-value-tech={isTechnical(item.label, item.value)}
            class:meta-value-missing={isMissing(item.value)}
            >{item.value}</span
          >
        </div>
      {/each}
    </div>
    <div class="description-row">
      <span class="meta-label">Description</span>
      <span class="meta-value" class:meta-value-missing={isMissing(description)}
        >{description}</span
      >
    </div>
  </section>
</section>

<style>
  .metadata-tech {
    font-family: var(--font-family);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .meta-section {
    padding: 0;
  }

  .meta-section + .meta-section {
    margin-top: 6px;
    padding-top: 10px;
    position: relative;
  }

  .meta-section + .meta-section::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 1px;
    background: rgba(148, 163, 184, 0.45);
  }

  .section-title {
    margin: 0 0 8px;
    color: #334155;
    font-size: 0.82rem;
    font-variant-caps: all-small-caps;
    letter-spacing: 0.08em;
    font-weight: 700;
  }

  .operation-columns {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }

  .structure-columns {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }

  .field {
    min-width: 0;
    padding: 6px 9px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid rgba(148, 163, 184, 0.14);
  }

  .meta-label {
    display: block;
    color: #6b7280;
    font-size: 0.67rem;
    font-variant-caps: all-small-caps;
    letter-spacing: 0.07em;
    font-weight: 700;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .meta-value {
    display: block;
    margin-top: 2px;
    color: #0b1220;
    font-size: 0.85rem;
    font-weight: 600;
    line-height: 1.35;
    white-space: normal;
    overflow-wrap: anywhere;
    word-break: break-word;
  }

  .meta-value-tech {
    font-family: inherit;
    font-size: 0.85rem;
    letter-spacing: 0;
  }

  .meta-value-missing {
    color: #cbd5e1;
    font-style: italic;
    font-weight: 500;
    font-family: inherit;
  }

  .description-row {
    margin-top: 8px;
    padding: 8px 9px 6px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid rgba(148, 163, 184, 0.14);
  }

</style>
