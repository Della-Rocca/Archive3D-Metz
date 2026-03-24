<script lang="ts">
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/auth";
  import { invoke } from "@tauri-apps/api/core";

  let password = "";
  let error = "";
  let loading = false;

  async function loginAsGuest() {
    authStore.login("guest");
    goto("/depot");
  }

  interface AppConfig {
    depot_path: string;
    validation_path: string;
    archive_path: string;
    settings_path: string;
  }

  async function loginAsAdmin() {
    if (!password.trim()) {
      error = "Veuillez entrer le mot de passe administrateur.";
      return;
    }

    loading = true;
    error = "";

    try {
      const valid = await invoke<boolean>("verify_admin_password", { password });
      if (valid) {
        authStore.login("admin");
        
        // Vérification si premier lancement (chemins manquants)
        try {
          const config = await invoke<AppConfig>("get_app_config");
          const needsSetup = !config.depot_path || !config.validation_path || !config.archive_path || !config.settings_path;
          if (needsSetup) {
            goto("/setup");
          } else {
            goto("/depot");
          }
        } catch (e) {
          console.error("Erreur vérification config :", e);
          goto("/depot"); // Fallback
        }
      } else {
        error = "Mot de passe incorrect.";
      }
    } catch (e) {
      error = `Erreur de connexion : ${e}`;
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      loginAsAdmin();
    }
  }
</script>

<div class="login-container">
  <main class="login-surface">
    <header class="login-header">
      <div class="brand-mark" aria-hidden="true"></div>
      <h1>Archive Metz</h1>
      <p class="subtitle">Accès administrateur pour valider et archiver, ou accès invité pour consulter.</p>
    </header>

    <section class="login-modes">
      <section class="login-panel admin-panel">
        <div class="panel-head">
          <h2>Administrateur</h2>
          <p>Validation, archivage et paramètres avancés.</p>
        </div>

        <div class="input-group">
          <label class="meta-label login-label" for="admin-password">Mot de passe</label>
          <input
            id="admin-password"
            type="password"
            class="meta-input"
            placeholder="Mot de passe administrateur"
            bind:value={password}
            on:keydown={handleKeydown}
            disabled={loading}
            autocomplete="current-password"
          />
        </div>

        {#if error}
          <div class="status-message error">{error}</div>
        {/if}

        <button class="btn btn-primary full-width login-primary-btn" on:click={loginAsAdmin} disabled={loading}>
          {loading ? "Connexion..." : "Se connecter"}
        </button>
      </section>

      <div class="modes-divider" aria-hidden="true"><span>ou</span></div>

      <section class="login-panel guest-panel">
        <div class="panel-head">
          <h2>Invité</h2>
          <p>Consultation des archives et dépôt de structures sans privilèges admin.</p>
        </div>
        <p class="guest-note">Accès immédiat, sans mot de passe.</p>
        <button class="btn btn-secondary full-width login-guest-btn" on:click={loginAsGuest}
          >Entrer en invité</button
        >
      </section>
    </section>
  </main>
</div>

<style>
  .login-container {
    min-height: 100dvh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-lg);
    background:
      radial-gradient(920px 500px at 0% -18%, rgba(30, 58, 95, 0.12), transparent 68%),
      radial-gradient(780px 440px at 100% 120%, rgba(142, 168, 202, 0.16), transparent 70%),
      linear-gradient(180deg, #f8fafc 0%, #edf2f8 100%);
  }

  .login-surface {
    width: 100%;
    max-width: 980px;
    background: rgba(255, 255, 255, 0.88);
    backdrop-filter: blur(16px) saturate(120%);
    -webkit-backdrop-filter: blur(16px) saturate(120%);
    border: 1px solid rgba(148, 163, 184, 0.3);
    border-radius: 24px;
    box-shadow: 0 24px 60px rgba(15, 23, 42, 0.12);
    overflow: hidden;
    padding: var(--spacing-2xl);
    display: grid;
    gap: var(--spacing-lg);
  }

  .login-modes {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: stretch;
    gap: var(--spacing-md);
  }

  .login-header {
    display: grid;
    justify-items: center;
    text-align: center;
    gap: 10px;
    margin-bottom: 4px;
  }

  .brand-mark {
    width: 36px;
    height: 36px;
    border-radius: 999px;
    background: linear-gradient(145deg, #1e3a5f 0%, #3f6aa2 100%);
    box-shadow:
      inset 0 0 0 5px rgba(255, 255, 255, 0.24),
      0 6px 14px rgba(30, 58, 95, 0.2);
  }

  .login-header h1 {
    margin: 0;
    color: var(--color-neutral-900);
    font-size: clamp(2.05rem, 3vw, 2.75rem);
    line-height: 1.18;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .subtitle {
    margin: 0;
    font-size: 0.92rem;
    line-height: 1.4;
    color: var(--color-neutral-700);
    max-width: 48ch;
  }

  .login-panel {
    border: 1px solid rgba(148, 163, 184, 0.28);
    background: rgba(255, 255, 255, 0.9);
    border-radius: 16px;
    padding: var(--spacing-lg);
    display: grid;
    gap: var(--spacing-md);
    align-content: start;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.92),
      0 10px 24px rgba(15, 23, 42, 0.04);
  }

  .admin-panel {
    border-color: rgba(30, 58, 95, 0.3);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.95) 0%, rgba(247, 251, 255, 0.92) 100%);
  }

  .guest-panel {
    background: linear-gradient(180deg, rgba(248, 250, 252, 0.78) 0%, rgba(255, 255, 255, 0.9) 100%);
  }

  .modes-divider {
    width: 42px;
    display: grid;
    align-items: center;
    justify-items: center;
    align-self: stretch;
  }

  .modes-divider::before {
    content: "";
    width: 1px;
    background: linear-gradient(180deg, transparent 0%, rgba(148, 163, 184, 0.5) 18%, rgba(148, 163, 184, 0.5) 82%, transparent 100%);
    height: 100%;
    grid-area: 1 / 1;
  }

  .modes-divider span {
    grid-area: 1 / 1;
    width: 32px;
    height: 32px;
    border-radius: 999px;
    border: 1px solid var(--color-neutral-400);
    background: rgba(255, 255, 255, 0.96);
    color: var(--color-neutral-600);
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 10px rgba(15, 23, 42, 0.08);
  }

  .panel-head {
    display: grid;
    gap: 4px;
  }

  .panel-head h2 {
    margin: 0;
    color: var(--color-neutral-900);
    font-size: var(--font-size-base);
    line-height: 1.2;
  }

  .panel-head p {
    margin: 0;
    color: var(--color-neutral-700);
    font-size: 0.86rem;
    line-height: 1.4;
  }

  .guest-note {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-neutral-600);
  }

  .input-group {
    display: grid;
    gap: 6px;
  }

  .login-label {
    margin-bottom: 0;
    font-size: 0.69rem;
  }

  .input-group .meta-input {
    min-height: 40px;
  }

  .full-width {
    width: 100%;
  }

  .login-primary-btn,
  .login-guest-btn {
    min-height: 42px;
    border-radius: 12px;
  }

  .status-message {
    margin: 0;
    text-align: left;
  }

  @media (max-width: 880px) {
    .login-surface {
      max-width: 620px;
    }

    .login-modes {
      grid-template-columns: 1fr;
      gap: var(--spacing-lg);
    }

    .modes-divider {
      width: 100%;
      height: 16px;
      display: flex;
      align-items: center;
      justify-content: center;
      position: relative;
    }

    .modes-divider::before,
    .modes-divider::after {
      height: 1px;
      width: 100%;
      background: linear-gradient(90deg, transparent 0%, rgba(148, 163, 184, 0.5) 16%, rgba(148, 163, 184, 0.5) 84%, transparent 100%);
    }

    .modes-divider span {
      position: absolute;
    }
  }

  @media (max-width: 640px) {
    .login-container {
      padding: var(--spacing-md);
    }

    .login-surface {
      padding: var(--spacing-lg);
    }

    .login-panel {
      padding: var(--spacing-md);
    }
  }
</style>
