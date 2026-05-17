<script lang="ts">
  import type { Snippet } from "svelte";
  import type { AppAction } from "./app-data";

  let {
    appName,
    appSubtitle,
    actions,
    activeAction,
    userName,
    userDetail,
    planLabel,
    onSelectAction,
    onSelectManagement,
    children
  }: {
    appName: string;
    appSubtitle: string;
    actions: AppAction[];
    activeAction: string;
    userName: string;
    userDetail: string;
    planLabel: string;
    onSelectAction: (id: string) => void;
    onSelectManagement: (id: "subscription" | "config" | "user") => void;
    children?: Snippet;
  } = $props();

  const userInitial = $derived((userName || "?").slice(0, 1).toUpperCase());
</script>

<div class="desktop-shell">
  <aside class="desktop-sidebar" aria-label="{appName} navigation">
    <div class="desktop-brand">
      <div class="desktop-brand-mark" aria-hidden="true">{appName.slice(0, 1)}</div>
      <div class="desktop-brand-copy">
        <strong>{appName}</strong>
        <span>{appSubtitle}</span>
      </div>
    </div>

    <nav class="desktop-actions" aria-label="Actions and functionality">
      {#each actions as action}
        <button
          type="button"
          class="desktop-action"
          class:active={activeAction === action.id}
          onclick={() => onSelectAction(action.id)}
        >
          <span class="desktop-action-icon" aria-hidden="true">{action.icon}</span>
          <span class="desktop-action-copy">
            <strong>{action.label}</strong>
            <small>{action.detail}</small>
          </span>
        </button>
      {/each}
    </nav>

    <div class="desktop-management" aria-label="User management, config, and subscription">
      <button
        type="button"
        class="desktop-management-row"
        class:active={activeAction === "subscription"}
        onclick={() => onSelectManagement("subscription")}
      >
        <span>Subscription</span>
        <strong>{planLabel}</strong>
      </button>
      <button
        type="button"
        class="desktop-management-row"
        class:active={activeAction === "config"}
        onclick={() => onSelectManagement("config")}
      >
        <span>Config</span>
        <strong>Runtime</strong>
      </button>
      <button
        type="button"
        class="desktop-user"
        class:active={activeAction === "user"}
        onclick={() => onSelectManagement("user")}
      >
        <span class="desktop-avatar" aria-hidden="true">{userInitial}</span>
        <span class="desktop-user-copy">
          <strong>{userName}</strong>
          <small>{userDetail}</small>
        </span>
      </button>
    </div>
  </aside>

  <main class="desktop-main">
    {#if children}
      {@render children()}
    {/if}
  </main>
</div>
