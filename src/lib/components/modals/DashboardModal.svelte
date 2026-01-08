<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { authState } from "$lib/stores/auth.svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";

  let { isOpen = $bindable(false) } = $props();

  let stats = $state<any>(null);
  let logs = $state<any[]>([]);
  let isLoading = $state(false);

  // Вычисляемые свойства (Runes)
  let daysRegistered = $derived.by(() => {
    if (!stats?.user_created_at) return "N/A";
    const created = new Date(stats.user_created_at);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - created.getTime());
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    return `${diffDays} days`;
  });

  let timeSinceLastLogin = $derived.by(() => {
    if (!stats?.last_login) return "First session";
    const last = new Date(stats.last_login);
    const now = new Date();
    const diffMs = now.getTime() - last.getTime();

    const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const hours = Math.floor(
      (diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)
    );
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    const parts = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    parts.push(`${minutes}m ago`);

    return parts.join(" ");
  });

  $effect(() => {
    if (isOpen && authState.user?.user_id) {
      loadData();
    }
  });

  async function loadData() {
    isLoading = true;
    try {
      const [s, l] = await Promise.all([
        invoke("get_user_statistics", { userId: authState.user!.user_id }),
        invoke("get_app_user_logs", { userId: authState.user!.user_id }),
      ]);
      stats = s;
      logs = l as any[];
    } catch (e) {
      console.error("Failed to load dashboard data:", e);
    } finally {
      isLoading = false;
    }
  }

  // --- ГЕНЕРАЦИЯ ОТЧЕТОВ НА ФРОНТЕНДЕ ---

  function generateUserActivityReportText(logs: any[]): string {
    let report = "";
    report += "==================================================\n";
    report += "               PG MANAGER - USER REPORT           \n";
    report += "==================================================\n\n";

    // Header
    report +=
      "TIMESTAMP                 | ACTION               | DETAILS                       \n";
    report +=
      "--------------------------+----------------------+------------------------------\n";

    for (const log of logs) {
      const date = new Date(log.timestamp);
      // Форматируем дату как YYYY-MM-DD HH:mm:ss
      const timeStr = date.toISOString().replace("T", " ").substring(0, 19);

      const action = log.action_type || "";
      const details = log.details || "";
      const shortDetails =
        details.length > 30 ? details.substring(0, 27) + "..." : details;

      // Паддинг для колонок (аналог {:<25} в Rust)
      const col1 = timeStr.padEnd(25, " ");
      const col2 = action.padEnd(20, " ");
      const col3 = shortDetails.padEnd(30, " ");

      report += `${col1} | ${col2} | ${col3}\n`;
    }

    report += "\n==================================================\n";
    report += "                  END OF REPORT                   \n";
    report += "==================================================\n";
    return report;
  }

  function generateStatisticsReportText(stats: any): string {
    let report = "";
    report += "==================================================\n";
    report += "            PG MANAGER - STATISTICS REPORT        \n";
    report += "==================================================\n\n";

    report += `Report Generated: ${new Date().toLocaleString()}\n\n`;

    report += "--- General ---\n";
    report += `User: ${authState.user?.username}\n`;
    report += `Registered: ${stats.user_created_at ? new Date(stats.user_created_at).toLocaleString() : "N/A"}\n`;
    report += `Days Registered: ${daysRegistered}\n`;
    report += `Total Connections: ${stats.total_connections}\n`;
    report += `Total Saved Queries: ${stats.pinned_queries}\n\n`;

    report += "--- Query Execution ---\n";
    report += `Total Queries Executed: ${stats.total_queries}\n`;
    report += `Successful: ${stats.success_queries}\n`;
    report += `Failed: ${stats.error_queries}\n`;

    const rate =
      stats.total_queries > 0
        ? ((stats.success_queries / stats.total_queries) * 100).toFixed(1)
        : "0.0";
    report += `Success Rate: ${rate}%\n\n`;

    report += "--- Session Info ---\n";
    report += `Last Login (Previous): ${stats.last_login ? new Date(stats.last_login).toLocaleString() : "First Session"}\n`;
    report += `Time Since Last Login: ${timeSinceLastLogin}\n`;

    report += "\n==================================================\n";
    report += "                  END OF REPORT                   \n";
    report += "==================================================\n";
    return report;
  }

  async function downloadActivityReport() {
    try {
      const reportText = generateUserActivityReportText(logs);
      const filePath = await save({
        filters: [{ name: "Text Report", extensions: ["txt"] }],
        defaultPath: `activity_report_${authState.user?.username}.txt`,
      });
      if (filePath) {
        await writeTextFile(filePath, reportText);
        alert("Activity Report saved!");
      }
    } catch (e) {
      console.error(e);
      alert("Failed to save report: " + e);
    }
  }

  async function downloadStatsReport() {
    try {
      const reportText = generateStatisticsReportText(stats);
      const filePath = await save({
        filters: [{ name: "Text Report", extensions: ["txt"] }],
        defaultPath: `stats_report_${authState.user?.username}.txt`,
      });
      if (filePath) {
        await writeTextFile(filePath, reportText);
        alert("Statistics Report saved!");
      }
    } catch (e) {
      console.error(e);
      alert("Failed to save report: " + e);
    }
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div
      class="modal-box w-11/12 max-w-6xl h-[85vh] flex flex-col p-0 overflow-hidden bg-base-100"
    >
      <!-- Header -->
      <div
        class="p-6 border-b border-base-200 flex justify-between items-center bg-base-200/50"
      >
        <div>
          <h3 class="font-bold text-2xl flex items-center gap-2">
            Dashboard
            <span class="badge badge-primary badge-outline text-xs align-top"
              >User Stats</span
            >
          </h3>
          <p class="text-sm opacity-60 mt-1">
            Activity overview for <span class="font-bold"
              >{authState.user?.username}</span
            >
          </p>
        </div>
        <button
          class="btn btn-sm btn-circle btn-ghost"
          onclick={() => (isOpen = false)}>✕</button
        >
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        {#if isLoading}
          <div class="flex justify-center items-center h-full">
            <span class="loading loading-spinner loading-lg text-primary"
            ></span>
          </div>
        {:else if stats}
          <!-- Stat Cards Row 1: General Stats -->
          <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-4">
            <div class="stats shadow bg-base-200">
              <div class="stat">
                <div class="stat-figure text-primary">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    class="inline-block w-8 h-8 stroke-current"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M13 10V3L4 14h7v7l9-11h-7z"
                    ></path></svg
                  >
                </div>
                <div class="stat-title">Total Queries</div>
                <div class="stat-value text-primary">{stats.total_queries}</div>
                <div class="stat-desc">Executed in history</div>
              </div>
            </div>

            <div class="stats shadow bg-base-200">
              <div class="stat">
                <div class="stat-figure text-success">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    class="inline-block w-8 h-8 stroke-current"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                    ></path></svg
                  >
                </div>
                <div class="stat-title">Success Rate</div>
                <div class="stat-value text-success">
                  {stats.total_queries > 0
                    ? Math.round(
                        (stats.success_queries / stats.total_queries) * 100
                      )
                    : 0}%
                </div>
                <div class="stat-desc">
                  {stats.error_queries} failed queries
                </div>
              </div>
            </div>

            <div class="stats shadow bg-base-200">
              <div class="stat">
                <div class="stat-figure text-secondary">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    class="inline-block w-8 h-8 stroke-current"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 12h14M12 5l7 7-7 7"
                    ></path></svg
                  >
                </div>
                <div class="stat-title">Connections</div>
                <div class="stat-value text-secondary">
                  {stats.total_connections}
                </div>
                <div class="stat-desc">Active database links</div>
              </div>
            </div>

            <div class="stats shadow bg-base-200">
              <div class="stat">
                <div class="stat-title">Saved Queries</div>
                <div class="stat-value text-accent">{stats.pinned_queries}</div>
                <div class="stat-desc">Pinned items</div>
              </div>
            </div>
          </div>

          <!-- Stat Cards Row 2: Time Stats (Requested Changes) -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
            <div class="stats shadow bg-base-200">
              <div class="stat">
                <div class="stat-title">Registration Info</div>
                <div class="stat-value text-xl font-mono mt-1">
                  {stats.user_created_at
                    ? new Date(stats.user_created_at).toLocaleDateString()
                    : "N/A"}
                </div>
                <div class="stat-desc text-info font-bold mt-1">
                  Days registered: {daysRegistered}
                </div>
              </div>
            </div>

            <div class="stats shadow bg-base-200">
              <div class="stat">
                <div class="stat-title">Session Info</div>
                <div class="stat-value text-xl font-mono mt-1">
                  {stats.last_login
                    ? new Date(stats.last_login).toLocaleString()
                    : "First Session"}
                </div>
                <div class="stat-desc text-warning font-bold mt-1">
                  Time since last login: {timeSinceLastLogin}
                </div>
              </div>
            </div>
          </div>

          <!-- Activity Log Table -->
          <div class="flex justify-between items-center mb-4">
            <h4 class="font-bold text-lg opacity-80">Recent Activity Log</h4>
            <div class="flex gap-2">
              <button
                class="btn btn-outline btn-sm gap-2"
                onclick={downloadStatsReport}
              >
                📄 Download Stats
              </button>
              <button
                class="btn btn-primary btn-sm gap-2"
                onclick={downloadActivityReport}
              >
                📊 Download Activity Log
              </button>
            </div>
          </div>

          <div
            class="overflow-x-auto bg-base-200/30 rounded-box border border-base-300"
          >
            <table class="table table-xs table-pin-rows">
              <thead>
                <tr>
                  <th>Timestamp</th>
                  <th>Action Type</th>
                  <th>Details</th>
                </tr>
              </thead>
              <tbody>
                {#each logs as log}
                  <tr class="hover">
                    <td class="font-mono opacity-70 w-48"
                      >{new Date(log.timestamp).toLocaleString()}</td
                    >
                    <td>
                      <span
                        class="badge badge-sm
                        {log.action_type === 'LOGIN'
                          ? 'badge-success'
                          : log.action_type === 'QUERY_ERROR'
                            ? 'badge-error'
                            : log.action_type === 'EXECUTE_QUERY'
                              ? 'badge-info'
                              : 'badge-ghost'}"
                      >
                        {log.action_type}
                      </span>
                    </td>
                    <td class="truncate max-w-md opacity-80" title={log.details}
                      >{log.details || "-"}</td
                    >
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>

      <div class="p-4 border-t border-base-200 bg-base-200/50 flex justify-end">
        <button class="btn" onclick={() => (isOpen = false)}>Close</button>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop">
      <button onclick={() => (isOpen = false)}>close</button>
    </form>
  </div>
{/if}
