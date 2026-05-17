export type AppPanel = {
  kicker: string;
  headline: string;
  description: string;
  stats: Array<{ label: string; value: string }>;
  points: Array<{ title: string; detail: string }>;
};

export type AppAction = {
  id: string;
  label: string;
  detail: string;
  icon: string;
  panel: AppPanel;
};

export type AppData = {
  appName: string;
  appSubtitle: string;
  userName: string;
  userDetail: string;
  planLabel: string;
  actions: AppAction[];
  management: {
    subscription: AppPanel;
    config: AppPanel;
    user: AppPanel;
  };
};

export const appData: AppData = {
  appName: "Fathom",
  appSubtitle: "SPARC analysis",
  userName: "Kenneth",
  userDetail: "Personal workspace",
  planLabel: "Free plan",
  actions: [
    {
      id: "filings",
      label: "Filings",
      detail: "10-K analysis",
      icon: "F",
      panel: {
        kicker: "Actions and functionality",
        headline: "Analyze filings without early collapse.",
        description:
          "Fathom keeps financial filing perspectives separate until promoted facts survive the gate. The desktop layer gives analysts a place to inspect the run.",
        stats: [
          { label: "Corpus", value: "EDGAR" },
          { label: "Gate", value: "Converge" },
          { label: "Mode", value: "SPARC" }
        ],
        points: [
          { title: "Disclosure risk", detail: "Compare risk factors, MD&A tone, auditor language, and restatements." },
          { title: "Provenance", detail: "Promoted facts keep source, query, snapshot, and suggestor context." },
          { title: "Analyst review", detail: "Disagreements are surfaced for human review instead of hidden in a summary." }
        ]
      }
    },
    {
      id: "suggestors",
      label: "Suggestors",
      detail: "Perspective lanes",
      icon: "S",
      panel: {
        kicker: "Formation",
        headline: "Run narrow financial perspectives.",
        description:
          "Each suggestor proposes a specific fact. The shell reserves space for drift, segment, auditor, restatement, and insider-activity lanes.",
        stats: [
          { label: "Lanes", value: "5-10" },
          { label: "Facts", value: "Gated" },
          { label: "Replay", value: "Deterministic" }
        ],
        points: [
          { title: "Risk factor drift", detail: "Track count and language changes between filings." },
          { title: "Segment truth", detail: "Compare segment narrative against reported numbers." },
          { title: "Disagreement map", detail: "Preserve conflicting signals until review." }
        ]
      }
    },
    {
      id: "portfolio",
      label: "Portfolio",
      detail: "Screen holdings",
      icon: "P",
      panel: {
        kicker: "Scale",
        headline: "Screen a portfolio with repeatable facts.",
        description:
          "The desktop surface is structured for analyst workflows that compare filings across holdings, quarters, and acquisition candidates.",
        stats: [
          { label: "Screen", value: "Batch" },
          { label: "Storage", value: "Lake" },
          { label: "Output", value: "Facts" }
        ],
        points: [
          { title: "Portfolio screen", detail: "Run the same formation across many companies." },
          { title: "M&A screen", detail: "Focus on hidden write-down, working-capital, and disclosure risk." },
          { title: "Earnings prep", detail: "Rank questions by surprise and evidence quality." }
        ]
      }
    }
  ],
  management: {
    subscription: {
      kicker: "Subscription",
      headline: "Free plan",
      description: "Plan controls are present in the shared lower management area for later billing integration.",
      stats: [
        { label: "Plan", value: "Free" },
        { label: "Workspace", value: "1" },
        { label: "Billing", value: "None" }
      ],
      points: [
        { title: "Entitlements", detail: "Placeholder for corpus scale, seats, and provider budgets." },
        { title: "Billing", detail: "Ready for a common Marquee subscription surface." }
      ]
    },
    config: {
      kicker: "Config",
      headline: "Desktop runtime configuration.",
      description: "Local Tauri/Svelte layer for Fathom, with future hooks into filings, lakehouse queries, and runs.",
      stats: [
        { label: "Port", value: "5110" },
        { label: "Runtime", value: "Tauri" },
        { label: "Frontend", value: "Svelte" }
      ],
      points: [
        { title: "Data source", detail: "Reserved for EDGAR, Iceberg, and local fixture settings." },
        { title: "Run bridge", detail: "Ready to invoke SPARC workflows from the desktop." }
      ]
    },
    user: {
      kicker: "User management",
      headline: "Personal analyst workspace.",
      description: "Account and workspace controls sit below the functional app navigation, matching Inkling and Wolfgang.",
      stats: [
        { label: "User", value: "Kenneth" },
        { label: "Role", value: "Owner" },
        { label: "Scope", value: "Local" }
      ],
      points: [
        { title: "Identity", detail: "Placeholder for analyst profile and organization membership." },
        { title: "Workspace", detail: "Personal filing workspace until shared workspaces are added." }
      ]
    }
  }
};
