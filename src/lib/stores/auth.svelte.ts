import { invoke } from "@tauri-apps/api/core";

// Тип, соответствующий модели Rust
export interface AppUser {
  user_id: number;
  username: string;
  role: string | null;
  created_at?: string;
}

let isAuthenticated = $state(false);
let user = $state<AppUser | null>(null);
let error = $state<string | null>(null);

let verifyUserCredentials = async (
  username: string,
  password: string
): Promise<AppUser | null> => {
  let result = await invoke<AppUser>("verify_user_credentials", {
    username: username,
    password: password,
  }).catch((err) => {
    console.error(err);
    return null;
  });

  return result || null;
};

export const authState = {
  get isAuthenticated() {
    return isAuthenticated;
  },
  get user() {
    return user;
  },
  login: async (username: string, password: string) => {
    if (!username || !password) {
      error = "Invalid username or password";
      return;
    }

    const result = await verifyUserCredentials(username, password);

    if (!result) {
      error = "Invalid username or password";
      return;
    }

    isAuthenticated = true;

    user = result;
    error = null;
  },
  logout: () => {
    isAuthenticated = false;
    user = null;
    error = null;
  },
  get error() {
    return error;
  },
};
