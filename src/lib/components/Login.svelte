<script lang="ts">
  import { authState } from "$lib/stores/auth.svelte";

  let username = $state("");
  let password = $state("");
  let isLoading = $state(false);

  async function handleLogin() {
    if (!username || !password) return;
    isLoading = true;
    try {
      await authState.login(username, password);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="hero min-h-screen bg-base-200">
  <div class="hero-content flex-col lg:flex-row-reverse gap-8">
    <div class="text-center lg:text-left max-w-sm">
      <h1 class="text-5xl font-bold text-primary">pg-manager</h1>
      <p class="py-6">Simple PostgreSQL database management tool.</p>
    </div>
    <div class="card shrink-0 w-full max-w-sm shadow-2xl bg-base-100">
      <form
        class="card-body flex flex-col items-center"
        onsubmit={(e) => {
          e.preventDefault();
          handleLogin();
        }}
      >
        <div class="form-control">
          <label class="label">
            <span class="label-text">Username</span>
          </label>
          <input
            type="text"
            placeholder="Username"
            class="input input-bordered"
            bind:value={username}
            required
          />
        </div>
        <div class="form-control">
          <label class="label">
            <span class="label-text">Password</span>
          </label>
          <input
            type="password"
            placeholder="Password"
            class="input input-bordered"
            bind:value={password}
            required
          />
        </div>

        {#if authState.error}
          <div class="alert alert-error text-xs shadow-lg mt-2">
            <span>{authState.error}</span>
          </div>
        {/if}

        <div class="form-control mt-6">
          <button class="btn btn-primary" disabled={isLoading}>
            {#if isLoading}
              <span class="loading loading-spinner"></span>
            {/if}
            Login
          </button>
        </div>
      </form>
    </div>
  </div>
</div>
