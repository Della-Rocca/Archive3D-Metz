# Svelte 5 Runes Migration Design

**Date:** 2026-07-16  
**Project:** Archive3D-Metz (Tauri desktop app)  
**Scope:** Migrate Svelte 4 reactive patterns to Svelte 5 runes. SvelteKit stays unchanged.

---

## Context

Project runs SvelteKit 2 + Svelte 5.0.0 but uses zero runes — all state is Svelte 4 style (`writable`, `derived`, `let` + `$:`). Migration adopts idiomatic Svelte 5 patterns without touching routing, Tauri integration, or Three.js code.

---

## Architecture

No architectural change. Same file-based routing, same Tauri/Rust backend, same store structure. Only the reactive primitives change:

| Before | After |
|--------|-------|
| `writable(x)` | `$state(x)` |
| `derived(store, fn)` | `$derived(expr)` |
| `$: computed = expr` | `let computed = $derived(expr)` |
| `$: { sideEffect }` | `$effect(() => { sideEffect })` |
| `$storeName` (auto-subscribe) | direct property access |

---

## Components

### 1. Auth Store

**File:** `src/lib/stores/auth.ts` → **rename** to `src/lib/stores/auth.svelte.ts`

Runes require `.svelte.ts` extension for module-level reactive state.

```ts
// auth.svelte.ts
const permissions = {
  guest: { canViewArchive: true, canDeposit: true, canValidate: false, canEditMetadata: false, canAccessConfig: false },
  admin: { canViewArchive: true, canDeposit: true, canValidate: true, canEditMetadata: true, canAccessConfig: true },
}

let _role = $state<'guest' | 'admin'>('guest')

export const auth = {
  get role() { return _role },
  login(role: 'guest' | 'admin') { _role = role },
  logout() { _role = 'guest' },
  can(action: keyof typeof permissions.guest): boolean {
    return permissions[_role][action]
  },
}
```

Consumers replace `import { authStore } from '$lib/stores/auth'` with `import { auth } from '$lib/stores/auth.svelte'` and replace `$authStore.role` with `auth.role`.

### 2. Layout (`+layout.svelte`)

- Auth guard: `$: if (!$authStore ...) goto(...)` → `$effect(() => { if (!auth.role ...) goto(...) })`
- Permission checks: `$authStore.role === 'admin'` → `auth.role === 'admin'`

### 3. Page Components (7 files)

Each page migrates its local state independently. Pattern is uniform:

```svelte
<!-- Before -->
<script lang="ts">
  let items = []
  let loading = false
  $: filtered = items.filter(x => x.active)
  $: { if (loading) doSomething() }
</script>

<!-- After -->
<script lang="ts">
  let items = $state([])
  let loading = $state(false)
  let filtered = $derived(items.filter(x => x.active))
  $effect(() => { if (loading) doSomething() })
</script>
```

### 4. Reusable Components (13 files)

Same pattern as pages. Most are self-contained with only local state — no store dependency beyond auth.

---

## Data Flow

No change. Tauri `invoke()` calls remain identical. Three.js integration unchanged. Only how Svelte tracks reactivity changes internally.

```
User Action → Component ($state mutation) → $derived recomputes → DOM updates
                                          → $effect triggers → Tauri invoke()
```

---

## Migration Order

1. `src/lib/stores/auth.svelte.ts` — shared dependency, migrate first
2. `src/routes/+layout.svelte` — global auth guard
3. `src/routes/archive/+page.svelte` — 3,799 lines, most complex
4. `src/routes/validation/+page.svelte` — 2,160 lines
5. `src/routes/depot/+page.svelte` — 1,306 lines
6. `src/routes/tutorial/+page.svelte` — 1,141 lines
7. `src/routes/+page.svelte`, `login/+page.svelte`, `setup/+page.svelte`, `settings/+page.svelte`
8. Reusable components in `src/lib/components/`

Each file is an independent commit. Svelte 5 supports old and new syntax in different files simultaneously, so partial migration is safe.

---

## Error Handling

- `$effect` runs after mount — auth redirects in `$effect` behave identically to `$:` reactive statements in layout context
- `$derived` is pure — no async, no side effects inside `$derived`
- Async Tauri calls stay in event handlers or `$effect`, not in `$derived`

---

## What Does NOT Change

- SvelteKit routing (`+page.svelte`, `+layout.svelte` filenames)
- `$app/navigation` (`goto`, `replaceState`)
- `$app/stores` (`$page`) — these are SvelteKit stores, not migrated
- Tauri `invoke()` calls
- Three.js scene management
- CSS / design tokens
- TypeScript types

---

## Success Criteria

- All 20 components compile without Svelte 4 deprecation warnings
- Auth store exports reactive object (not Svelte store) with `login`, `logout`, `can` methods
- Zero `writable`/`derived` imports from `svelte/store` in `.svelte` files
- App navigates, authenticates, and displays 3D viewer correctly after migration
