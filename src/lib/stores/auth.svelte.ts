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
