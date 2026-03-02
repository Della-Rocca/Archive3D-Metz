<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { authStore, can } from "$lib/stores/auth";

  // Routes publiques qui ne nécessitent pas d'authentification
  const publicRoutes = ['/login'];
  let topnavEl: HTMLElement | null = null;
  let topnavResizeObserver: ResizeObserver | null = null;
  let observedTopnavEl: HTMLElement | null = null;

  function syncTopnavHeight() {
    if (typeof document === "undefined") return;
    const topnavHeight =
      !isLoginPage && topnavEl ? Math.ceil(topnavEl.getBoundingClientRect().height) : 0;
    document.documentElement.style.setProperty("--app-topnav-height", `${topnavHeight}px`);
  }

  onMount(() => {
    // Vérifier si l'utilisateur est authentifié
    const unsubscribe = authStore.subscribe(state => {
      const currentPath = window.location.pathname;

      // Si pas authentifié et pas sur une route publique, rediriger vers login
      if (!state.isAuthenticated && !publicRoutes.includes(currentPath)) {
        goto('/login');
      }
    });

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
      unsubscribe();
      topnavResizeObserver?.disconnect();
      topnavResizeObserver = null;
      observedTopnavEl = null;
      window.removeEventListener("resize", syncTopnavHeight);
      document.documentElement.style.setProperty("--app-topnav-height", "0px");
    };
  });

  $: isLoginPage = $page.url.pathname === '/login';
  $: if (topnavResizeObserver && topnavEl !== observedTopnavEl) {
    if (observedTopnavEl) {
      topnavResizeObserver.unobserve(observedTopnavEl);
    }
    if (topnavEl) {
      topnavResizeObserver.observe(topnavEl);
    }
    observedTopnavEl = topnavEl;
  }
  $: syncTopnavHeight();

  // Stores dérivés pour les permissions (réactifs)
  const canValidateStore = can('canValidate');
  const canAccessConfigStore = can('canAccessConfig');

  $: canValidate = $canValidateStore;
  $: canAccessConfig = $canAccessConfigStore;
  $: currentRole = $authStore.role;

  function handleLogout() {
    authStore.logout();
  }
</script>

{#if !isLoginPage}
  <nav class="topnav" bind:this={topnavEl}>
    <div class="nav-brand">Archive Metz</div>
    <a href="/depot" class:selected={$page.url.pathname.startsWith("/depot")}>
      Dépôt
    </a>
    {#if canValidate}
      <a
        href="/validation"
        class:selected={$page.url.pathname.startsWith("/validation")}
      >
        Validation
      </a>
    {/if}
    <a
      href="/archive"
      class:selected={$page.url.pathname.startsWith("/archive")}
    >
      Archive
    </a>
    <div class="nav-user">
      {#if canAccessConfig}
        <a class="settings-btn" href="/settings" title="Paramètres">
          <svg class="settings-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 15C13.6569 15 15 13.6569 15 12C15 10.3431 13.6569 9 12 9C10.3431 9 9 10.3431 9 12C9 13.6569 10.3431 15 12 15Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M19.4 15C19.2669 15.3016 19.2272 15.6362 19.286 15.9606C19.3448 16.285 19.4995 16.5843 19.73 16.82L19.79 16.88C19.976 17.0657 20.1235 17.2863 20.2241 17.5291C20.3248 17.7719 20.3766 18.0322 20.3766 18.295C20.3766 18.5578 20.3248 18.8181 20.2241 19.0609C20.1235 19.3037 19.976 19.5243 19.79 19.71C19.6043 19.896 19.3837 20.0435 19.1409 20.1441C18.8981 20.2448 18.6378 20.2966 18.375 20.2966C18.1122 20.2966 17.8519 20.2448 17.6091 20.1441C17.3663 20.0435 17.1457 19.896 16.96 19.71L16.9 19.65C16.6643 19.4195 16.365 19.2648 16.0406 19.206C15.7162 19.1472 15.3816 19.1869 15.08 19.32C14.7842 19.4468 14.532 19.6572 14.3543 19.9255C14.1766 20.1938 14.0813 20.5082 14.08 20.83V21C14.08 21.5304 13.8693 22.0391 13.4942 22.4142C13.1191 22.7893 12.6104 23 12.08 23C11.5496 23 11.0409 22.7893 10.6658 22.4142C10.2907 22.0391 10.08 21.5304 10.08 21V20.91C10.0723 20.579 9.96512 20.258 9.77251 19.9887C9.5799 19.7194 9.31074 19.5143 9 19.4C8.69838 19.2669 8.36381 19.2272 8.03941 19.286C7.71502 19.3448 7.41568 19.4995 7.18 19.73L7.12 19.79C6.93425 19.976 6.71368 20.1235 6.47088 20.2241C6.22808 20.3248 5.96783 20.3766 5.705 20.3766C5.44217 20.3766 5.18192 20.3248 4.93912 20.2241C4.69632 20.1235 4.47575 19.976 4.29 19.79C4.10405 19.6043 3.95653 19.3837 3.85588 19.1409C3.75523 18.8981 3.70343 18.6378 3.70343 18.375C3.70343 18.1122 3.75523 17.8519 3.85588 17.6091C3.95653 17.3663 4.10405 17.1457 4.29 16.96L4.35 16.9C4.58054 16.6643 4.73519 16.365 4.794 16.0406C4.85282 15.7162 4.81312 15.3816 4.68 15.08C4.55324 14.7842 4.34276 14.532 4.07447 14.3543C3.80618 14.1766 3.49179 14.0813 3.17 14.08H3C2.46957 14.08 1.96086 13.8693 1.58579 13.4942C1.21071 13.1191 1 12.6104 1 12.08C1 11.5496 1.21071 11.0409 1.58579 10.6658C1.96086 10.2907 2.46957 10.08 3 10.08H3.09C3.42099 10.0723 3.742 9.96512 4.0113 9.77251C4.28059 9.5799 4.48572 9.31074 4.6 9C4.73312 8.69838 4.77282 8.36381 4.714 8.03941C4.65519 7.71502 4.50054 7.41568 4.27 7.18L4.21 7.12C4.02405 6.93425 3.87653 6.71368 3.77588 6.47088C3.67523 6.22808 3.62343 5.96783 3.62343 5.705C3.62343 5.44217 3.67523 5.18192 3.77588 4.93912C3.87653 4.69632 4.02405 4.47575 4.21 4.29C4.39575 4.10405 4.61632 3.95653 4.85912 3.85588C5.10192 3.75523 5.36217 3.70343 5.625 3.70343C5.88783 3.70343 6.14808 3.75523 6.39088 3.85588C6.63368 3.95653 6.85425 4.10405 7.04 4.29L7.1 4.35C7.33568 4.58054 7.63502 4.73519 7.95941 4.794C8.28381 4.85282 8.61838 4.81312 8.92 4.68H9C9.29577 4.55324 9.54802 4.34276 9.72569 4.07447C9.90337 3.80618 9.99872 3.49179 10 3.17V3C10 2.46957 10.2107 1.96086 10.5858 1.58579C10.9609 1.21071 11.4696 1 12 1C12.5304 1 13.0391 1.21071 13.4142 1.58579C13.7893 1.96086 14 2.46957 14 3V3.09C14.0013 3.41179 14.0966 3.72618 14.2743 3.99447C14.452 4.26276 14.7042 4.47324 15 4.6C15.3016 4.73312 15.6362 4.77282 15.9606 4.714C16.285 4.65519 16.5843 4.50054 16.82 4.27L16.88 4.21C17.0657 4.02405 17.2863 3.87653 17.5291 3.77588C17.7719 3.67523 18.0322 3.62343 18.295 3.62343C18.5578 3.62343 18.8181 3.67523 19.0609 3.77588C19.3037 3.87653 19.5243 4.02405 19.71 4.21C19.896 4.39575 20.0435 4.61632 20.1441 4.85912C20.2448 5.10192 20.2966 5.36217 20.2966 5.625C20.2966 5.88783 20.2448 6.14808 20.1441 6.39088C20.0435 6.63368 19.896 6.85425 19.71 7.04L19.65 7.1C19.4195 7.33568 19.2648 7.63502 19.206 7.95941C19.1472 8.28381 19.1869 8.61838 19.32 8.92V9C19.4468 9.29577 19.6572 9.54802 19.9255 9.72569C20.1938 9.90337 20.5082 9.99872 20.83 10H21C21.5304 10 22.0391 10.2107 22.4142 10.5858C22.7893 10.9609 23 11.4696 23 12C23 12.5304 22.7893 13.0391 22.4142 13.4142C22.0391 13.7893 21.5304 14 21 14H20.91C20.5882 14.0013 20.2738 14.0966 20.0055 14.2743C19.7372 14.452 19.5268 14.7042 19.4 15V15Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </a>
      {/if}
      <button type="button" class="user-badge" class:admin={currentRole === 'admin'}>
        {currentRole === 'admin' ? 'Admin' : 'Invité'}
      </button>
      <button type="button" class="logout-btn" on:click={handleLogout}>
        Déconnexion
      </button>
    </div>
  </nav>
{/if}

<slot />

<style>
  /* ---------- VARIABLES CSS (DESIGN SYSTEM) ---------- */
  :global(:root) {
    /* Couleurs principales */
    --color-primary: #1e3a5f;
    --color-primary-hover: #152d4a;
    --color-secondary: #c17a5c;
    --color-secondary-hover: #a66849;

    /* Couleurs neutres */
    --color-neutral-900: #2d3748;
    --color-neutral-800: #4a5568;
    --color-neutral-700: #718096;
    --color-neutral-600: #a0aec0;
    --color-neutral-500: #cbd5e0;
    --color-neutral-400: #e2e8f0;
    --color-neutral-300: #edf2f7;
    --color-neutral-200: #f7fafc;
    --color-neutral-100: #ffffff;

    /* Couleurs fonctionnelles */
    --color-success: #2f855a;
    --color-success-bg: #c6f6d5;
    --color-success-border: #9ae6b4;
    --color-warning: #d97706;
    --color-warning-bg: #fef3c7;
    --color-warning-border: #fde68a;
    --color-error: #c53030;
    --color-error-bg: #fed7d7;
    --color-error-border: #fc8181;
    --color-info: #3182ce;
    --color-info-bg: #dbeafe;
    --color-info-border: #93c5fd;

    /* Typographie */
    --font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --font-size-xs: 0.75rem;
    --font-size-sm: 0.875rem;
    --font-size-base: 0.9375rem;
    --font-size-lg: 1.125rem;
    --font-size-xl: 1.5rem;
    --font-size-2xl: 2rem;

    /* Espacements */
    --spacing-xs: 0.25rem;
    --spacing-sm: 0.5rem;
    --spacing-md: 0.75rem;
    --spacing-lg: 1rem;
    --spacing-xl: 1.5rem;
    --spacing-2xl: 2rem;

    /* Bordures */
    --border-radius-sm: 0.375rem;
    --border-radius-md: 0.5rem;
    --border-radius-lg: 0.75rem;
    --border-width: 1px;

    /* Ombres */
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
    --shadow-md: 0 2px 4px rgba(0, 0, 0, 0.08);
    --shadow-lg: 0 4px 12px rgba(0, 0, 0, 0.12);

    /* Layout */
    --app-topnav-height: 0px;
  }

  /* ---------- NAV ---------- */
  .topnav {
    position: sticky;
    top: 0;
    z-index: 1000;
    display: flex;
    gap: var(--spacing-md);
    padding: var(--spacing-md) var(--spacing-xl);
    border-bottom: var(--border-width) solid var(--color-neutral-400);
    background: var(--color-neutral-100);
    font-family: var(--font-family);
    align-items: center;
    flex-wrap: wrap;
    box-shadow: var(--shadow-sm);
  }

  .nav-brand {
    font-size: var(--font-size-lg);
    font-weight: 700;
    color: var(--color-primary);
    margin-right: var(--spacing-lg);
    letter-spacing: -0.02em;
  }

  .topnav a {
    text-decoration: none;
    font-size: var(--font-size-sm);
    font-weight: 500;
    padding: var(--spacing-sm) var(--spacing-lg);
    border-radius: var(--border-radius-md);
    color: var(--color-neutral-800);
    transition: all 0.2s ease;
  }

  .topnav a.selected {
    background: var(--color-primary);
    color: var(--color-neutral-100);
  }

  .topnav a:hover:not(.selected) {
    background: var(--color-neutral-300);
    color: var(--color-neutral-900);
  }

  .nav-user {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  /* Styles communs pour les 3 boutons */
  .user-badge,
  .settings-btn,
  .logout-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--border-radius-md);
    font-size: var(--font-size-sm);
    font-weight: 600;
    border: 1px solid var(--color-neutral-400);
    cursor: pointer;
    transition: all 0.2s;
    text-decoration: none;
  }

  .user-badge {
    background: var(--color-neutral-300);
    color: var(--color-neutral-900);
    cursor: default;
  }

  .user-badge.admin {
    background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
    color: #92400e;
    border-color: #fde68a;
  }

  .settings-btn {
    background: transparent;
    color: var(--color-neutral-700);
    width: 36px;
    height: 36px;
    padding: 0;
    border: none;
  }

  .settings-btn:hover {
    background: transparent;
    border: none;
    color: var(--color-primary);
  }

  .settings-btn:hover .settings-icon {
    animation: spin 0.6s ease-in-out;
  }

  .settings-icon {
    width: 18px;
    height: 18px;
    transition: transform 0.3s;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(180deg);
    }
  }

  .logout-btn {
    background: transparent;
    color: var(--color-neutral-800);
  }

  .logout-btn:hover {
    background: var(--color-error);
    border-color: var(--color-error);
    color: white;
  }

  /* ---------- STYLES GÉNÉRAUX ---------- */
  :global(body) {
    margin: 0;
    padding: 0;
    background: var(--color-neutral-200);
    color: var(--color-neutral-900);
    font-family: var(--font-family);
    font-size: var(--font-size-base);
    line-height: 1.6;
  }

  :global(main) {
    font-family: var(--font-family);
    max-width: 1400px;
    margin: 0 auto;
  }

  /* Labels */
  :global(label.meta-label) {
    display: block;
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--color-neutral-800);
    margin-bottom: var(--spacing-xs);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  /* Inputs et Selects */
  :global(input.meta-input),
  :global(textarea.meta-input),
  :global(select.meta-select),
  :global(input.field-input),
  :global(select.field-select) {
    width: 100%;
    border-radius: var(--border-radius-md);
    border: var(--border-width) solid var(--color-neutral-300);
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: var(--font-size-base);
    font-family: var(--font-family);
    box-sizing: border-box;
    background-color: var(--color-neutral-100);
    color: var(--color-neutral-900);
    transition: all 0.2s ease;
  }

  :global(input.meta-input:focus),
  :global(textarea.meta-input:focus),
  :global(select.meta-select:focus),
  :global(input.field-input:focus),
  :global(select.field-select:focus) {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(30, 58, 95, 0.12);
  }

  :global(input.meta-input[readonly]),
  :global(textarea.meta-input[readonly]) {
    background-color: var(--color-neutral-300);
    cursor: not-allowed;
    color: var(--color-neutral-700);
  }

  :global(textarea.meta-input) {
    min-height: 80px;
    resize: vertical;
    font-family: var(--font-family);
  }

  /* Wrappers pour selects */
  :global(.meta-select-wrapper) {
    position: relative;
  }

  :global(.meta-select-wrapper select.meta-select) {
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    padding-right: 2.5rem;
  }

  :global(.meta-select-wrapper::after) {
    content: "";
    position: absolute;
    right: var(--spacing-lg);
    top: 50%;
    transform: translateY(-50%);
    width: 14px;
    height: 14px;
    background-color: var(--color-neutral-700);
    mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath d='M6 9l6 6 6-6' fill='none' stroke='black' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    mask-repeat: no-repeat;
    mask-position: center;
    mask-size: contain;
    pointer-events: none;
    transition: transform 0.2s ease, background-color 0.2s ease;
  }

  :global(.meta-select-wrapper:focus-within::after) {
    transform: translateY(-50%) rotate(180deg);
    background-color: var(--color-primary);
  }

  /* Wrappers pour inputs avec datalist */
  :global(.meta-input-wrapper) {
    position: relative;
  }

  :global(.meta-input-wrapper input.meta-input) {
    padding-right: 2.5rem;
  }

  :global(.meta-input-wrapper::after) {
    content: "";
    position: absolute;
    right: var(--spacing-lg);
    top: 50%;
    transform: translateY(-50%);
    width: 14px;
    height: 14px;
    background-color: var(--color-neutral-700);
    mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath d='M6 9l6 6 6-6' fill='none' stroke='black' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    mask-repeat: no-repeat;
    mask-position: center;
    mask-size: contain;
    pointer-events: none;
    transition: transform 0.2s ease, background-color 0.2s ease;
  }

  :global(.meta-input-wrapper:focus-within::after) {
    transform: translateY(-50%) rotate(180deg);
    background-color: var(--color-primary);
  }

  /* Combobox custom globaux */
  :global(.combo) {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    max-width: 100%;
    border: var(--border-width) solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    background: var(--color-neutral-100);
    padding-right: 2.25rem;
    transition: border-color 0.2s, box-shadow 0.2s;
    box-sizing: border-box;
  }

  :global(.combo:focus-within) {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(30, 58, 95, 0.12);
  }

  :global(.combo-input) {
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    background: transparent;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    font-size: var(--font-size-base);
    font-family: var(--font-family);
    color: var(--color-neutral-900);
    box-sizing: border-box;
  }

  :global(.combo-input:focus) {
    outline: none;
  }

  :global(.combo-input:disabled),
  :global(.combo-input[readonly]) {
    background-color: transparent;
    cursor: not-allowed;
    color: var(--color-neutral-700);
  }

  :global(.combo-toggle) {
    position: absolute;
    right: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--color-neutral-700);
    font-size: 0.95rem;
    cursor: pointer;
  }

  :global(.combo-toggle:disabled) {
    opacity: 0.4;
    cursor: not-allowed;
  }

  :global(.combo:focus-within .combo-toggle) {
    color: var(--color-primary);
  }

  :global(.combo-list) {
    position: absolute;
    z-index: 1000;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    max-height: 220px;
    overflow-y: auto;
    background: var(--color-neutral-100);
    border: var(--border-width) solid var(--color-neutral-400);
    border-radius: var(--border-radius-md);
    box-shadow: var(--shadow-md);
    padding: 4px 0;
    margin: 0;
    list-style: none;
  }

  :global(.combo-option) {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 12px;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-900);
    cursor: pointer;
    border: none;
    background: transparent;
    font-family: var(--font-family);
  }

  :global(.combo-option:hover),
  :global(.combo-option:focus) {
    background: var(--color-neutral-300);
    outline: none;
  }

  /* Boutons globaux */
  :global(.btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-xl);
    border-radius: var(--border-radius-md);
    font-size: var(--font-size-sm);
    font-weight: 600;
    font-family: var(--font-family);
    cursor: pointer;
    border: none;
    transition: all 0.2s ease;
    text-decoration: none;
  }

  :global(.btn-primary) {
    background: var(--color-primary);
    color: var(--color-neutral-100);
  }

  :global(.btn-primary:hover:not(:disabled)) {
    background: var(--color-primary-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  :global(.btn-primary:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.btn-secondary) {
    background: var(--color-neutral-100);
    color: var(--color-neutral-900);
    border: var(--border-width) solid var(--color-neutral-400);
  }

  :global(.btn-secondary:hover:not(:disabled)) {
    background: var(--color-neutral-300);
    border-color: var(--color-neutral-500);
  }

  :global(.btn-outline) {
    background: transparent;
    color: var(--color-primary);
    border: var(--border-width) solid var(--color-primary);
  }

  :global(.btn-outline:hover:not(:disabled)) {
    background: var(--color-primary);
    color: var(--color-neutral-100);
  }

  /* Messages de statut globaux */
  :global(.status-message) {
    padding: var(--spacing-md) var(--spacing-lg);
    border-radius: var(--border-radius-md);
    font-size: var(--font-size-sm);
    border: var(--border-width) solid;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  :global(.status-message.success) {
    background: var(--color-success-bg);
    border-color: var(--color-success-border);
    color: var(--color-success);
  }

  :global(.status-message.error) {
    background: var(--color-error-bg);
    border-color: var(--color-error-border);
    color: var(--color-error);
  }

  :global(.status-message.warning) {
    background: var(--color-warning-bg);
    border-color: var(--color-warning-border);
    color: var(--color-warning);
  }

  :global(.status-message.info) {
    background: var(--color-info-bg);
    border-color: var(--color-info-border);
    color: var(--color-info);
  }
</style>
