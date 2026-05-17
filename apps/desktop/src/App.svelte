<script lang="ts">
  import DesktopShell from "./lib/DesktopShell.svelte";
  import { appData } from "./lib/app-data";

  let activeAction = $state(appData.actions[0]?.id ?? "config");

  const managementPanels = $derived([
    { id: "subscription", panel: appData.management.subscription },
    { id: "config", panel: appData.management.config },
    { id: "user", panel: appData.management.user }
  ]);

  const activePanel = $derived(
    appData.actions.find((action) => action.id === activeAction)?.panel ??
      managementPanels.find((item) => item.id === activeAction)?.panel ??
      appData.management.config
  );
</script>

<DesktopShell
  appName={appData.appName}
  appSubtitle={appData.appSubtitle}
  actions={appData.actions}
  {activeAction}
  userName={appData.userName}
  userDetail={appData.userDetail}
  planLabel={appData.planLabel}
  onSelectAction={(id) => (activeAction = id)}
  onSelectManagement={(id) => (activeAction = id)}
>
  <section class="workspace">
    <header class="workspace-header">
      <div>
        <p>{activePanel.kicker}</p>
        <h1>{activePanel.headline}</h1>
      </div>
      <span>{appData.appName} desktop</span>
    </header>

    <section class="workspace-body">
      <div class="overview-panel">
        <p>{activePanel.description}</p>
        <div class="stat-grid">
          {#each activePanel.stats as stat}
            <div class="stat-card">
              <strong>{stat.value}</strong>
              <span>{stat.label}</span>
            </div>
          {/each}
        </div>
      </div>

      <div class="point-grid">
        {#each activePanel.points as point}
          <article>
            <h2>{point.title}</h2>
            <p>{point.detail}</p>
          </article>
        {/each}
      </div>
    </section>
  </section>
</DesktopShell>
