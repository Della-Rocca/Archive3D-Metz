import { writable, derived, get } from 'svelte/store';
import { goto } from '$app/navigation';

export type UserRole = 'guest' | 'admin';

export interface AuthState {
  isAuthenticated: boolean;
  role: UserRole | null;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
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
    },

    hasPermission(requiredRole: UserRole): boolean {
      let currentState: AuthState = { isAuthenticated: false, role: null };
      subscribe(state => currentState = state)();

      if (!currentState.isAuthenticated) return false;
      if (requiredRole === 'guest') return true;
      if (requiredRole === 'admin') return currentState.role === 'admin';
      return false;
    }
  };
}

export const authStore = createAuthStore();

// Permissions par rôle
export const permissions = {
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
