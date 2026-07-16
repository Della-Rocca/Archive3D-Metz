# Svelte 5 Runes Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the Archive3D-Metz codebase from Svelte 4 reactive patterns (`writable`, `derived`, `$:`) to Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`), keeping SvelteKit intact.

**Architecture:** Create a new `auth.svelte.ts` with module-level `$state`, then migrate each component and page to runes mode. Svelte 5 supports mixing legacy and runes files, so migration is safe file-by-file. Migrating a component to runes mode requires converting all `export let` props to `$props()` in that same file.

**Tech Stack:** SvelteKit 2, Svelte 5, TypeScript, Tauri 2.

## Global Constraints

- SvelteKit stays: do NOT touch routing files (`+layout.ts`, `svelte.config.js`), `$app/navigation`, `$app/stores`
- Tauri `invoke()` calls: do NOT change
- Three.js code in `Archive3DViewer.svelte`: do NOT touch Three.js logic, only reactive declarations
- `$page` from `$app/stores` is a SvelteKit store — keep using `$page` with `$` prefix, do NOT migrate it
- `onMount`, `onDestroy`, `tick` from `svelte` remain valid in runes mode — keep where appropriate
- `createEventDispatcher` is deprecated in runes mode but still compiles — keep for now (out of scope to migrate to callback props)
- File rename: `auth.ts` → `auth.svelte.ts` (runes require `.svelte.ts` extension for module-level `$state`)
- Each task ends with a commit of only that task's files

## Transformation Reference

| Svelte 4 | Svelte 5 Runes |
|----------|---------------|
| `writable(x)` | `let _ = $state(x)` (module-level) |
| `derived(store, fn)` | Plain function reading `$state` |
| `export let x = v` | `let { x = v } = $props()` |
| `export let x` (bindable) | `let { x = $bindable() } = $props()` |
| `let x = v` (local reactive) | `let x = $state(v)` |
| `$: y = expr` | `let y = $derived(expr)` |
| `$: { sideEffect }` | `$effect(() => { sideEffect })` |
| `$: if (cond) { effect }` | `$effect(() => { if (cond) { effect } })` |
| `store.subscribe(fn)` in `onMount` | `$effect(() => { ...reads from $state... })` |
| `$storeName` (auto-subscribe) | Direct property access on reactive object |
| `touched = touched` (force update) | Remove — `$state` deep-tracks mutations |

---

## Task 1: Create `src/lib/stores/auth.svelte.ts`

**Files:**
- Create: `src/lib/stores/auth.svelte.ts`
- Keep: `src/lib/stores/auth.ts` (delete in Task 21 after all consumers migrated)

**Interfaces:**
- Produces: `auth` object with `isAuthenticated: boolean`, `role: UserRole | null`, `login(role)`, `logout()` methods; `can(action)` plain function returning `boolean`

- [ ] **Step 1: Create the new auth store file**

```typescript
// src/lib/stores/auth.svelte.ts
import { goto } from '$app/navigation';

export type UserRole = 'guest' | 'admin';

export interface AuthState {
  isAuthenticated: boolean;
  role: UserRole | null;
}

const permissions = {
  guest: {
    canViewArchive: true,
    canDeposit: true,
    canValidate: false,
    canEditMetadata: false,
    canAccessConfig: false,
  },
  admin: {
    canViewArchive: true,
    canDeposit: true,
    canValidate: true,
    canEditMetadata: true,
    canAccessConfig: true,
  },
} as const;

let _state = $state<AuthState>({ isAuthenticated: false, role: null });

export const auth = {
  get isAuthenticated() { return _state.isAuthenticated; },
  get role() { return _state.role; },

  login(role: UserRole) {
    _state = { isAuthenticated: true, role };
  },

  logout() {
    _state = { isAuthenticated: false, role: null };
    goto('/login');
  },
};

export function can(action: keyof typeof permissions.guest): boolean {
  if (!_state.isAuthenticated || !_state.role) return false;
  return permissions[_state.role][action];
}
```

- [ ] **Step 2: Verify file compiles**

Run: `npx tsc --noEmit`  
Expected: No errors in `auth.svelte.ts`

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/auth.svelte.ts
git commit -m "feat: add Svelte 5 runes auth store (auth.svelte.ts)"
```

---

## Task 2: Migrate `src/routes/+layout.svelte`

**Files:**
- Modify: `src/routes/+layout.svelte`

**Interfaces:**
- Consumes: `auth`, `can` from `$lib/stores/auth.svelte`

- [ ] **Step 1: Update the script block**

Replace the entire `<script lang="ts">` block (lines 1–74) with:

```svelte
<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount, onDestroy } from "svelte";
  import { auth, can } from "$lib/stores/auth.svelte";

  const publicRoutes = ['/login'];
  let topnavEl: HTMLElement | null = $state(null);
  let topnavResizeObserver: ResizeObserver | null = $state(null);
  let observedTopnavEl: HTMLElement | null = $state(null);

  function syncTopnavHeight() {
    if (typeof document === "undefined") return;
    const topnavHeight =
      !hideTopnav && topnavEl ? Math.ceil(topnavEl.getBoundingClientRect().height) : 0;
    document.documentElement.style.setProperty("--app-topnav-height", `${topnavHeight}px`);
  }

  // Auth guard — re-runs whenever auth.isAuthenticated changes
  $effect(() => {
    if (!auth.isAuthenticated && !publicRoutes.includes(window.location.pathname)) {
      goto('/login');
    }
  });

  // ResizeObserver setup — runs once on mount
  onMount(() => {
    syncTopnavHeight();
    if (typeof ResizeObserver !== "undefined") {
      topnavResizeObserver = new ResizeObserver(() => syncTopnavHeight());
      if (topnavEl) {
        topnavResizeObserver.observe(topnavEl);
        observedTopnavEl = topnavEl;
      }
    } else {
      window.addEventListener("resize", syncTopnavHeight);
    }
    return () => {
      topnavResizeObserver?.disconnect();
      topnavResizeObserver = null;
      observedTopnavEl = null;
      window.removeEventListener("resize", syncTopnavHeight);
      document.documentElement.style.setProperty("--app-topnav-height", "0px");
    };
  });

  // Re-observe when topnavEl binding updates after first render
  $effect(() => {
    if (topnavResizeObserver && topnavEl !== observedTopnavEl) {
      if (observedTopnavEl) topnavResizeObserver.unobserve(observedTopnavEl);
      if (topnavEl) topnavResizeObserver.observe(topnavEl);
      observedTopnavEl = topnavEl;
    }
  });

  // Sync height whenever hideTopnav or topnavEl changes
  $effect(() => {
    syncTopnavHeight();
  });

  let hideTopnav = $derived($page.url.pathname === '/login' || $page.url.pathname === '/setup');
  let canValidate = $derived(can('canValidate'));
  let canAccessConfig = $derived(can('canAccessConfig'));
  let currentRole = $derived(auth.role);

  function handleLogout() {
    auth.logout();
  }
</script>
```

- [ ] **Step 2: Run type check and launch app**

```bash
npx tsc --noEmit
```
Expected: No errors. Then launch the Tauri dev app and verify: login redirects unauthenticated users, nav bar shows/hides admin items correctly, logout works.

- [ ] **Step 3: Commit**

```bash
git add src/routes/+layout.svelte
git commit -m "feat: migrate +layout.svelte to Svelte 5 runes"
```

---

## Task 3: Migrate `src/routes/+page.svelte` (root redirect)

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Replace script block**

```svelte
<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth.svelte";

  $effect(() => {
    if (auth.isAuthenticated) {
      goto("/depot");
    } else {
      goto("/login");
    }
  });
</script>
```

- [ ] **Step 2: Verify in app**

Launch app. Navigating to `/` should redirect to `/login` (unauthenticated) or `/depot` (authenticated).

- [ ] **Step 3: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: migrate root page redirect to Svelte 5 runes"
```

---

## Task 4: Migrate `src/routes/tutorial/+page.svelte`

**Files:**
- Modify: `src/routes/tutorial/+page.svelte`

- [ ] **Step 1: Replace script block**

```svelte
<script lang="ts">
  import { auth } from "$lib/stores/auth.svelte";

  type SectionId = "intro" | "depot" | "archive" | "validation" | "settings" | "metadata";

  let activeSection: SectionId = $state("intro");
  let isAdmin = $derived(auth.role === "admin");

  const guestSections: { id: SectionId; label: string; icon: string }[] = [
    { id: "intro", label: "Introduction", icon: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" },
    { id: "depot", label: "Dépôt", icon: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" },
    { id: "archive", label: "Archive", icon: "M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8l1 12a2 2 0 002 2h8a2 2 0 002-2l1-12" },
  ];

  const adminSections: { id: SectionId; label: string; icon: string }[] = [
    { id: "validation", label: "Validation", icon: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" },
    { id: "settings", label: "Paramètres", icon: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" },
    { id: "metadata", label: "Métadonnées", icon: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" },
  ];

  let allSections = $derived(isAdmin ? [...guestSections, ...adminSections] : guestSections);
</script>
```

- [ ] **Step 2: Verify**

In app, navigate to `/tutorial`. Verify sidebar shows admin sections only when logged in as admin.

- [ ] **Step 3: Commit**

```bash
git add src/routes/tutorial/+page.svelte
git commit -m "feat: migrate tutorial page to Svelte 5 runes"
```

---

## Task 5: Migrate `src/routes/login/+page.svelte`

**Files:**
- Modify: `src/routes/login/+page.svelte`

- [ ] **Step 1: Update script imports and local state**

In the `<script lang="ts">` block, make these changes:

```svelte
<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth.svelte";   // was: { authStore }
  import { invoke } from "@tauri-apps/api/core";

  let password = $state("");
  let error = $state("");
  let loading = $state(false);

  async function loginAsGuest() {
    auth.login("guest");                             // was: authStore.login("guest")
    goto("/depot");
  }

  // ... rest unchanged, but replace authStore.login with auth.login
```

Replace every `authStore.login(` → `auth.login(` in this file.

- [ ] **Step 2: Verify**

Guest login and admin login both work and redirect correctly.

- [ ] **Step 3: Commit**

```bash
git add src/routes/login/+page.svelte
git commit -m "feat: migrate login page to Svelte 5 runes"
```

---

## Task 6: Migrate `src/routes/setup/+page.svelte`

**Files:**
- Modify: `src/routes/setup/+page.svelte`

- [ ] **Step 1: Update script**

Replace top of script block:

```svelte
<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth.svelte";   // was: { authStore }

  // ... keep interfaces and stepsInfo const unchanged ...

  let config: AppConfig = $state({
    depot_path: "",
    validation_path: "",
    archive_path: "",
    settings_path: "",
  });

  let step = $state(0);
  let saving = $state(false);
  let error = $state("");

  let currentField = $derived(
    stepsInfo[step].field as "depot_path" | "validation_path" | "archive_path" | "settings_path" | undefined
  );
  let isCurrentFieldFilled = $derived(currentField ? !!config[currentField] : false);

  onMount(async () => {
    if (auth.role !== "admin") {           // was: $authStore.role
      goto("/login");
      return;
    }
    try {
      const existing = await invoke<AppConfig>("get_app_config");
      config = { ...existing };
    } catch (e) {
      console.warn("Could not load initial config or is default", e);
    }
  });

  // ... rest of functions unchanged ...
</script>
```

- [ ] **Step 2: Verify**

Navigate to `/setup` (as admin). Wizard steps work, path selection functions.

- [ ] **Step 3: Commit**

```bash
git add src/routes/setup/+page.svelte
git commit -m "feat: migrate setup page to Svelte 5 runes"
```

---

## Task 7: Migrate `src/lib/components/ComboInput.svelte`

**Files:**
- Modify: `src/lib/components/ComboInput.svelte`

**Note:** `value` must be `$bindable()` — parents use `bind:value={...}`.

- [ ] **Step 1: Update script block**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let {
    value = $bindable(""),
    options = [],
    placeholder = "",
    disabled = false,
  }: {
    value?: string;
    options?: string[];
    placeholder?: string;
    disabled?: boolean;
  } = $props();

  const dispatch = createEventDispatcher();

  let open = $state(false);
  let inputEl: HTMLInputElement;
  let containerEl: HTMLDivElement;

  let filtered = $derived(options);

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
  }
</script>
```

Keep the rest of the template (`<div class="combo-wrap">` etc.) unchanged.

- [ ] **Step 2: Verify**

In app, open depot page. ComboInput fields (operation code, site etc.) autocomplete and dispatch correctly.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ComboInput.svelte
git commit -m "feat: migrate ComboInput to Svelte 5 runes"
```

---

## Task 8: Migrate `src/lib/components/MetadataEditor.svelte`

**Files:**
- Modify: `src/lib/components/MetadataEditor.svelte`

**Note:** No `export let` props — this component has no props, only local state and dispatched events.

- [ ] **Step 1: Update script block state declarations**

Replace the state variable declarations and `$:` lines (lines 66–96):

```svelte
  let presets: Presets = $state({
    operations: [],
    structure_types: [],
    operation_types: [],
    software_types: [],
    sites: [],
    responsables: [],
    model_authors: [],
    depositors: [],
  });

  let activeTab: "categories" | "operations" = $state("categories");
  let activeCategory: StringCategory = $state("structure_types");
  let newValue = $state("");
  let saving = $state(false);
  let loading = $state(true);
  let status = $state("");
  let statusType: "success" | "error" | "" = $state("");

  let newOp: OperationMeta = $state({
    code: "",
    site: "",
    op_type: "",
    responsable: "",
  });

  let newValueError = $derived(
    activeCategory === "sites" ? getSafeSegmentError(newValue) : ""
  );
  let newOpCodeError = $derived(getSafeSegmentError(newOp.code));
  let newOpSiteError = $derived(getSafeSegmentError(newOp.site));
```

Keep all function bodies and template unchanged.

- [ ] **Step 2: Verify**

Open Settings → MetadataEditor. Tabs, add/remove items, operations CRUD all work.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/MetadataEditor.svelte
git commit -m "feat: migrate MetadataEditor to Svelte 5 runes"
```

---

## Task 9: Migrate `src/lib/components/DepositCard.svelte`

**Files:**
- Modify: `src/lib/components/DepositCard.svelte`

**Note:** Props-only component (no local reactive state, no `$:` statements).

- [ ] **Step 1: Replace `export let` with `$props()`**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let {
    title = "",
    operationCode = "—",
    operationSite = "—",
    structureType = "—",
    hasModel = false,
    hasPhotos = false,
    hasOrtho = false,
    hasWork = false,
    active = false,
    revisionTagged = false,
  }: {
    title?: string;
    operationCode?: string;
    operationSite?: string;
    structureType?: string;
    hasModel?: boolean;
    hasPhotos?: boolean;
    hasOrtho?: boolean;
    hasWork?: boolean;
    active?: boolean;
    revisionTagged?: boolean;
  } = $props();

  const dispatch = createEventDispatcher();
</script>
```

Keep template unchanged.

- [ ] **Step 2: Verify**

Validation page shows deposit cards correctly.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/DepositCard.svelte
git commit -m "feat: migrate DepositCard to Svelte 5 runes"
```

---

## Task 10: Migrate `src/lib/components/domain/depot/DepositStepper.svelte`

**Files:**
- Modify: `src/lib/components/domain/depot/DepositStepper.svelte`

**Note:** Props-only component (no local reactive state, no `$:` statements). No props are bound from parent.

- [ ] **Step 1: Replace `export let` with `$props()`**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let {
    steps = [],
    currentStep = 1,
    step1Valid = false,
    step2Valid = false,
    step3Valid = false,
  }: {
    steps?: { id: number; label: string; desc: string }[];
    currentStep?: number;
    step1Valid?: boolean;
    step2Valid?: boolean;
    step3Valid?: boolean;
  } = $props();

  const dispatch = createEventDispatcher();

  function stepStatus(stepId: number): "completed" | "current" | "upcoming" | "error" {
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
```

- [ ] **Step 2: Verify**

Depot stepper renders correctly and step navigation works.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/domain/depot/DepositStepper.svelte
git commit -m "feat: migrate DepositStepper to Svelte 5 runes"
```

---

## Task 11: Migrate `src/lib/components/domain/depot/DepositStepOperation.svelte`

**Files:**
- Modify: `src/lib/components/domain/depot/DepositStepOperation.svelte`

**Note:** `operation` must be `$bindable()` — parent uses `bind:operation={operation}`.

- [ ] **Step 1: Update script block**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { OperationMeta, Presets } from "$lib/types/deposit";
  import { getSafeSegmentError, isSafeSegment } from "$lib/utils/path";

  let {
    operation = $bindable<OperationMeta>({ code: "", site: "", op_type: "", responsable: "" }),
    presets,
    errors = {},
    canAdmin = false,
  }: {
    operation?: OperationMeta;
    presets: Presets;
    errors?: Record<string, string>;
    canAdmin?: boolean;
  } = $props();

  const dispatch = createEventDispatcher<{ operationCreated: OperationMeta }>();

  function handleOperationSelect(e: Event) {
    const code = (e.target as HTMLSelectElement).value;
    const found = presets.operations.find((op) => op.code === code);
    if (found) {
      operation = { ...found };
    }
  }

  let showQuickCreate = $state(false);
  let quickSaving = $state(false);
  let quickError = $state("");
  let quickSuccess = $state(false);

  let quickForm: OperationMeta = $state({
    code: "",
    site: "",
    op_type: "",
    responsable: "",
  });

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

  let quickFormValid = $derived(
    quickForm.code.trim().length > 0 &&
    quickForm.site.trim().length > 0 &&
    isSafeSegment(quickForm.code) &&
    isSafeSegment(quickForm.site)
  );
  let quickCodeError = $derived(getSafeSegmentError(quickForm.code));
  let quickSiteError = $derived(getSafeSegmentError(quickForm.site));

  // ... rest of async functions unchanged ...
</script>
```

Keep template unchanged.

- [ ] **Step 2: Verify**

Depot Step 1 (Operation): selecting from presets and quick-create both work.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/domain/depot/DepositStepOperation.svelte
git commit -m "feat: migrate DepositStepOperation to Svelte 5 runes"
```

---

## Task 12: Migrate `src/lib/components/domain/depot/DepositStepStructure.svelte`

**Files:**
- Modify: `src/lib/components/domain/depot/DepositStepStructure.svelte`

**Note:** `structure` must be `$bindable()` — parent uses `bind:structure={structure}`.

- [ ] **Step 1: Update script block**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import ComboInput from "$lib/components/ComboInput.svelte";
  import type { StructureMeta, Presets } from "$lib/types/deposit";

  let {
    structure = $bindable<StructureMeta>({ id: "", st_type: "", description: "", model_author: "", depositor: "", photos_count: "", faces_count: "", software: "" }),
    presets,
    errors = {},
    countingPolygons = false,
  }: {
    structure?: StructureMeta;
    presets: Presets;
    errors?: Record<string, string>;
    countingPolygons?: boolean;
  } = $props();

  let selectedSoftware: string[] = $state([]);

  $effect(() => {
    if (structure.software) {
      const split = structure.software.split(", ").filter((s) => s);
      const current = selectedSoftware.join(", ");
      if (current !== structure.software) {
        selectedSoftware = split;
      }
    } else {
      selectedSoftware = [];
    }
  });

  const dispatch = createEventDispatcher();

  // ... rest of functions unchanged ...
</script>
```

- [ ] **Step 2: Verify**

Depot Step 2 (Structure): metadata fields and software multi-select work.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/domain/depot/DepositStepStructure.svelte
git commit -m "feat: migrate DepositStepStructure to Svelte 5 runes"
```

---

## Task 13: Migrate `src/lib/components/domain/depot/DepositStepRecap.svelte`

**Files:**
- Modify: `src/lib/components/domain/depot/DepositStepRecap.svelte`

- [ ] **Step 1: Update script block**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { OperationMeta, StructureMeta } from "$lib/types/deposit";

  let {
    operation,
    structure,
    filesCounts,
    recapIssues = [],
  }: {
    operation: OperationMeta;
    structure: StructureMeta;
    filesCounts: { model: number; ortho: number; photo: number; work: number };
    recapIssues?: string[];
  } = $props();

  const dispatch = createEventDispatcher();

  let totalFilesCount = $derived(
    filesCounts.model + filesCounts.ortho + filesCounts.photo + filesCounts.work
  );

  function editStep(step: number) {
    dispatch("edit", step);
  }
</script>
```

- [ ] **Step 2: Verify**

Depot Step 4 (Recap): total file count displays correctly.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/domain/depot/DepositStepRecap.svelte
git commit -m "feat: migrate DepositStepRecap to Svelte 5 runes"
```

---

## Task 14: Migrate `src/lib/components/domain/depot/FileDropZone.svelte`

**Files:**
- Modify: `src/lib/components/domain/depot/FileDropZone.svelte`

**Note:** No props need `$bindable()` — parent passes `files` and `dragOver` as read-only props, receives updates via events.

- [ ] **Step 1: Update script block**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { filename } from "$lib/utils/file";

  let {
    title,
    required = false,
    files = [],
    dropId,
    dragOver = null,
    error = "",
  }: {
    title: string;
    required?: boolean;
    files?: string[];
    dropId: string;
    dragOver?: string | null;
    error?: string;
  } = $props();

  const dispatch = createEventDispatcher();

  let isActive = $derived(dragOver === dropId);

  function handleRemove(index: number) {
    if (index >= 0 && index < files.length) {
      dispatch("remove", index);
    }
  }
</script>
```

- [ ] **Step 2: Verify**

Depot file drop zones accept files via drop and button, remove works.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/domain/depot/FileDropZone.svelte
git commit -m "feat: migrate FileDropZone to Svelte 5 runes"
```

---

## Task 15: Migrate `src/lib/components/archive/Archive3DViewer.svelte`

**Files:**
- Modify: `src/lib/components/archive/Archive3DViewer.svelte`

**Note:** Three.js logic, event listeners, and `onMount`/`onDestroy` bodies are untouched. Only reactive declarations change.

- [ ] **Step 1: Update props and reactive state at top of script**

Replace `export let` declarations and `let` reactive vars (lines 7–23), and replace the two `$:` lines (lines 213–217):

```svelte
<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { tick } from "svelte";
  import { onDestroy, onMount } from "svelte";
  import * as THREE from "three";

  let {
    modelPath = null,
    noVisualAssets = false,
  }: {
    modelPath?: string | null;
    noVisualAssets?: boolean;
  } = $props();

  // DOM / Three.js refs — not displayed in template, plain let is fine
  let viewerShellEl: HTMLElement | null = null;
  let mountEl: HTMLDivElement | null = null;
  let renderer: THREE.WebGLRenderer | null = null;
  let camera: THREE.PerspectiveCamera | null = null;
  let scene: THREE.Scene | null = null;
  let controls: { update: () => void; dispose: () => void } | null = null;
  let rootModel: THREE.Object3D | null = null;
  let frameId = 0;
  let disposed = false;

  // Reactive state (displayed in template or used in $effect)
  let mounted = $state(false);
  let lastInputSignature = $state("");
  let isExpanded = $state(false);
  let status: "idle" | "loading" | "error" | "empty" = $state("idle");
  let statusMessage = $state("Sélectionnez une structure pour afficher son modèle 3D.");

  // ... (keep all functions and onMount/onDestroy unchanged)

  // Replace the two $: lines with:
  let inputSignature = $derived(`${modelPath ?? ""}::${noVisualAssets ? "1" : "0"}`);
  $effect(() => {
    if (mounted && inputSignature !== lastInputSignature) {
      lastInputSignature = inputSignature;
      refreshFromProps();
    }
  });
</script>
```

- [ ] **Step 2: Verify**

Archive page: selecting a structure loads its 3D model. Model switches when selecting a different structure.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/archive/Archive3DViewer.svelte
git commit -m "feat: migrate Archive3DViewer to Svelte 5 runes"
```

---

## Task 16: Migrate `src/lib/components/archive/ArchiveMetadataPanel.svelte`

**Files:**
- Modify: `src/lib/components/archive/ArchiveMetadataPanel.svelte`

**Note:** No local reactive state. Props-only component.

- [ ] **Step 1: Update script block**

```svelte
<script lang="ts">
  interface MetadataDisplayItem {
    label: string;
    value: string;
  }

  let {
    operationItems = [],
    structureItems = [],
    description = "—",
  }: {
    operationItems?: MetadataDisplayItem[];
    structureItems?: MetadataDisplayItem[];
    description?: string;
  } = $props();

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
```

- [ ] **Step 2: Verify**

Archive page metadata panel shows operation and structure info when a model is selected.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/archive/ArchiveMetadataPanel.svelte
git commit -m "feat: migrate ArchiveMetadataPanel to Svelte 5 runes"
```

---

## Task 17: Migrate `src/routes/depot/+page.svelte`

**Files:**
- Modify: `src/routes/depot/+page.svelte`

**Reactive changes in this file:**

`$:` to convert:
```svelte
// BEFORE
const canEditMetadataStore = can("canEditMetadata");
$: canEditMetadata = $canEditMetadataStore;

$: fieldErrors = { ... };

$: filesValid = findInvalidFiles(modelFiles).length === 0 && ...;

$: step1Valid = !!operation.code.trim() && ...;
$: step2Valid = !!structure.id.trim() && ...;
$: step3Valid = modelFiles.length > 0 && filesValid;
$: allValid = step1Valid && step2Valid && step3Valid;

// AFTER
let canEditMetadata = $derived(can("canEditMetadata"));

let fieldErrors = $derived({ ... });  // same expression, no change to body

let filesValid = $derived(
  findInvalidFiles(modelFiles).length === 0 &&
  findInvalidFiles(orthoFiles).length === 0 &&
  findInvalidFiles(photoFiles).length === 0 &&
  findInvalidFiles(workFiles).length === 0
);

let step1Valid = $derived(!!operation.code.trim() && !!operation.site.trim() && isSafeSegment(operation.code) && isSafeSegment(operation.site));
let step2Valid = $derived(!!structure.id.trim() && !!structure.st_type.trim() && isSafeSegment(structure.id));
let step3Valid = $derived(modelFiles.length > 0 && filesValid);
let allValid = $derived(step1Valid && step2Valid && step3Valid);
```

- [ ] **Step 1: Update import**

Change: `import { can } from "$lib/stores/auth";`  
To: `import { can } from "$lib/stores/auth.svelte";`

- [ ] **Step 2: Convert all top-level `let` declarations to `$state`**

Rule: every `let x = value` at script root that holds mutable UI state → `let x = $state(value)`.

These include (non-exhaustive): `currentStep`, `operation`, `structure`, `modelFiles`, `orthoFiles`, `photoFiles`, `workFiles`, `presets`, `status`, `statusType`, `loading`, `countingPolygons`, `showMetadataEditor`, `touched`.

- [ ] **Step 3: Convert `$:` to `$derived`/`$effect`**

Apply transforms shown above. The `can("canEditMetadata")` store pattern becomes a single `$derived` line.

- [ ] **Step 4: Remove manual reactivity trigger**

Find the `touch()` function:
```ts
function touch(field: string) {
  touched[field] = true;
  touched = touched;  // ← REMOVE this line
}
```
With `$state`, object mutations are tracked automatically. The `touched = touched` reassignment is not needed.

- [ ] **Step 5: Verify**

Full deposit flow: fill operation → structure → files → recap → submit. Validation errors appear on blur/advance.

- [ ] **Step 6: Commit**

```bash
git add src/routes/depot/+page.svelte
git commit -m "feat: migrate depot page to Svelte 5 runes"
```

---

## Task 18: Migrate `src/routes/validation/+page.svelte`

**Files:**
- Modify: `src/routes/validation/+page.svelte`

**`$:` transforms:**

```svelte
// BEFORE
$: if (editMetadata) {
  editMetadata.structure.software = selectedSoftware.join(", ");
}

$: editFieldErrors = editMetadata ? { ... } : { ... };

$: editMetadataValid = !!editMetadata && ...;

// AFTER
$effect(() => {
  if (editMetadata) {
    editMetadata.structure.software = selectedSoftware.join(", ");
  }
});

let editFieldErrors = $derived(editMetadata ? { ... } : { ... });

let editMetadataValid = $derived(
  !!editMetadata &&
  !!editMetadata.operation.code.trim() &&
  !!editMetadata.operation.site.trim() &&
  !!editMetadata.structure.id.trim() &&
  isSafeSegment(editMetadata.operation.code) &&
  isSafeSegment(editMetadata.operation.site) &&
  isSafeSegment(editMetadata.structure.id)
);
```

- [ ] **Step 1: Convert all top-level `let` to `$state`**

All mutable `let` vars: `items`, `selectedItem`, `metadata`, `presets`, `loading`, `processing`, `status`, `statusType`, `filterType`, `filterOperation`, `filterRevision`, `searchQuery`, `listDetails`, `showPreviewModal`, `showArchiveConfirm`, `isEditing`, `editMetadata`, `selectedSoftware`, `activeTab`, `historyEntries`, `loadingHistory`, `resettingHistory`, `revisionTags`, `showCompactDetails`, `previewBodyEl`.

- [ ] **Step 2: Convert `$:` lines**

Apply the three transforms shown above.

- [ ] **Step 3: Verify**

Validation page: list loads, selecting a structure shows details, editing metadata and saving works.

- [ ] **Step 4: Commit**

```bash
git add src/routes/validation/+page.svelte
git commit -m "feat: migrate validation page to Svelte 5 runes"
```

---

## Task 19: Migrate `src/routes/archive/+page.svelte`

**Files:**
- Modify: `src/routes/archive/+page.svelte`

**`$:` transforms:**

```svelte
// BEFORE
$: totalPages = Math.ceil(total / perPage) || 1;

// AFTER
let totalPages = $derived(Math.ceil(total / perPage) || 1);
```

- [ ] **Step 1: Convert all top-level `let` to `$state`**

All mutable `let` vars displayed in template or used in `$derived`/`$effect`:
`activeTab`, `items`, `total`, `page`, `perPage`, `query`, `loading`, `consultationError`, `filterAsset`, `filterOperationCode`, `filterStructureType`, `filterDateOrder`, `operationFilterOptions`, `structureTypeFilterOptions`, `sortField`, `sortDirection`, `selectedItem`, `selectedDetails`, `detailLoading`, `detailError`, `detailActionStatus`, `detailRequestToken`, `orthoPreviewSrc`, `orthoPreviewPath`, `orthoPreviewLoading`, `orthoPreviewError`, `showOrthoPreviewModal`, `stats`, `loadingStats`, `statsError`, `recentActivity`, `loadingActivity`, `activityError`, `inventory`, `loadingInventory`, `inventoryError`, `inventoryQuery`, `inventorySortField`, `inventorySortDirection`, all `inventoryFilter*` vars, `activeInventoryFilterMenu`, `inventoryFilterMenuPos`, `inventoryPage`, `inventoryPerPage`, `inventoryExportStatus`, `inventoryExportStatusType`, `inventoryTableContainerEl`, `canScrollInventoryLeft`, `canScrollInventoryRight`, `showAllOperations`, `showAllStructureTypes`, `showAllAuthors`, `showAllYears`.

**Exception — keep as plain `let` (timer/RAF refs, not in template):**
- `consultationDebounceTimer`
- `inventoryScrollRaf`
- `inventoryExportStatusTimer`

- [ ] **Step 2: Convert `$:` line**

```svelte
let totalPages = $derived(Math.ceil(total / perPage) || 1);
```

- [ ] **Step 3: Verify**

Archive page: consultation tab loads items, filtering and pagination work, 3D viewer opens, inventory tab works, stats tab shows charts.

- [ ] **Step 4: Commit**

```bash
git add src/routes/archive/+page.svelte
git commit -m "feat: migrate archive page to Svelte 5 runes"
```

---

## Task 20: Migrate `src/routes/settings/+page.svelte`

**Files:**
- Modify: `src/routes/settings/+page.svelte`

**`$:` transforms:**

```svelte
// BEFORE
$: pathFieldIssues = computeFieldIssues(pathValidation);
$: globalValidationErrors = (pathValidation?.errors ?? []).filter(...);
$: globalValidationWarnings = (pathValidation?.warnings ?? []).filter(...);
$: validationErrorCount = pathValidation?.errors.length ?? 0;
$: validationWarningCount = pathValidation?.warnings.length ?? 0;
$: validationIsHealthy = validationErrorCount === 0 && validationWarningCount === 0;
$: configuredPathCount = pathFieldMeta.reduce(...);
$: pathSignature = pathFieldMeta.map((entry) => config[entry.field]).join("|");
$: if (readyForLiveValidation && pathSignature !== undefined) {
  scheduleLiveValidation();
}

// AFTER
let pathFieldIssues = $derived(computeFieldIssues(pathValidation));
let globalValidationErrors = $derived((pathValidation?.errors ?? []).filter((message) => !mapMessageToField(message)));
let globalValidationWarnings = $derived((pathValidation?.warnings ?? []).filter((message) => !mapMessageToField(message)));
let validationErrorCount = $derived(pathValidation?.errors.length ?? 0);
let validationWarningCount = $derived(pathValidation?.warnings.length ?? 0);
let validationIsHealthy = $derived(validationErrorCount === 0 && validationWarningCount === 0);
let configuredPathCount = $derived(pathFieldMeta.reduce((count, entry) => {
  const field = entry.field;
  const hasValue = config[field].trim().length > 0;
  const hasNoErrors = pathFieldIssues[field].errors.length === 0;
  return count + (hasValue && hasNoErrors ? 1 : 0);
}, 0));
let pathSignature = $derived(pathFieldMeta.map((entry) => config[entry.field]).join("|"));
$effect(() => {
  if (readyForLiveValidation && pathSignature !== undefined) {
    scheduleLiveValidation();
  }
});
```

- [ ] **Step 1: Convert all top-level `let` to `$state`**

All mutable: `config`, `savedConfig`, `pathValidation`, `saving`, `saveError`, `saveSuccess`, `validationDebounceTimer` (plain `let` — not in template), `readyForLiveValidation`, `activePathField`.

- [ ] **Step 2: Convert `$:` lines**

Apply transforms shown above.

- [ ] **Step 3: Verify**

Settings page: config loads, path pickers work, live validation fires on change, save/reset works.

- [ ] **Step 4: Commit**

```bash
git add src/routes/settings/+page.svelte
git commit -m "feat: migrate settings page to Svelte 5 runes"
```

---

## Task 21: Delete `src/lib/stores/auth.ts`

**Files:**
- Delete: `src/lib/stores/auth.ts`

- [ ] **Step 1: Confirm no remaining imports**

Search for any remaining `from "$lib/stores/auth"` (without `.svelte`) across all source files. There should be zero.

```bash
grep -r 'stores/auth"' src/
```

Expected output: empty (no results).

- [ ] **Step 2: Delete the old store**

```bash
rm src/lib/stores/auth.ts
```

- [ ] **Step 3: Final type check**

```bash
npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 4: Full app smoke test**

Launch app. Test:
1. `/` redirects to `/login` when unauthenticated
2. Guest login → lands on `/depot`, no admin nav items
3. Admin login → nav shows Validation + Settings gear
4. Depot: multi-step form completes
5. Archive: structures load, 3D viewer opens
6. Logout → redirects to `/login`

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: complete Svelte 5 runes migration — remove legacy auth.ts"
```
