import { writable, derived } from 'svelte/store';
import { goto } from '$app/navigation';

export type UserRole = 'guest' | 'admin';

export interface AuthState {
  isAuthenticated: boolean;
  role: UserRole | null;
}

function createAuthStore() {
  const { subscribe, set } = writable<AuthState>({
    isAuthenticated: false,
    role: null
  });

  return {
    subscribe,

    login(role: UserRole) {
      set({
        isAuthenticated: true,
        role
      });
    },

    logout() {
      set({
        isAuthenticated: false,
        role: null
      });
      goto('/login');
    }
  };
}

export const authStore = createAuthStore();

// Permissions par rôle
const permissions = {
  guest: {
    canViewArchive: true,
    canDeposit: true,
    canValidate: false,
    canEditMetadata: false,
    canAccessConfig: false
  },
  admin: {
    canViewArchive: true,
    canDeposit: true,
    canValidate: true,
    canEditMetadata: true,
    canAccessConfig: true
  }
};

// Store dérivé pour vérifier les permissions de manière réactive
export function can(action: keyof typeof permissions.guest) {
  return derived(authStore, $authStore => {
    if (!$authStore.isAuthenticated || !$authStore.role) return false;
    return permissions[$authStore.role][action];
  });
}
