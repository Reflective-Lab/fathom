---
source: llm
type: doctrine
---
# Platform Feedback Loop

This KB belongs to a marquee application. A marquee application is not a generic downstream consumer of Bedrock, Mosaic, Organism, Axiom, Converge, or Helms. It is a proof surface that should grow fast as a real product while making the shared platform stronger.

The application may move quickly, but it should not become a parallel platform. When it discovers reusable machinery, the result should flow back into the correct lower layer instead of being reimplemented locally.

## Core Rule

The app owns domain meaning. The platform owns reusable machinery.

This app should own:

- product UX, workflows, routes, and operator experience
- vertical language, examples, personas, fixtures, and truth catalogs
- domain projections and writeback semantics
- trust posture, consent language, and human review moments for the product domain
- first proof loops that expose whether Bedrock and Mosaic surfaces are strong enough

This app should not own reusable platform cores:

- fixed-point loops, authority promotion, or fact convergence
- TruthDocument compilation or generic intent-packet semantics
- formation selection or reusable formation orchestration
- reusable policy, memory, analytics, solver, provider, storage, vector, or connector engines
- duplicate regression, fuzzy logic, optimization, retrieval, authorization, or adapter frameworks

## Platform Routing

Route reusable pressure to the owner that already exists:

| Capability | Owner | App posture |
|---|---|---|
| `.truths`, typed truths, domain vocabulary, intent packets | Axiom | author domain catalogs and consume compiled intent packets |
| formation selection, formation compile/run lifecycle, extension choice | Organism | request and inspect formations, do not build local formation engines |
| fixed-point loops, promotion, facts, convergence receipts | Converge | use convergence receipts and feedback, do not fork convergence mechanics |
| host UX, HITL review, app-facing mediation, writeback policy | Helms | integrate through host/app contracts |
| specialist implementation bench | Mosaic extensions | select, configure, and compose specialists; do not recreate their cores |

## Mosaic Specialist Bench

Formation and application design should consider the full Mosaic bench before inventing local logic:

| Specialist | Use when the app needs |
|---|---|
| Arbiter | authorization, policy, delegation, approval requirements, Cedar-backed decisions |
| Manifold | provider adapters, generic LLM/tool/feed/search/fetch/storage/vector surfaces |
| Embassy | source-specific connectors and external-system diplomacy |
| Mnemos | memory, recall, knowledge retrieval, prior episodes, evidence seeding, learning feedback |
| Prism | regression, fuzzy inference, ranking, forecasting, anomaly detection, classification, analytic critique |
| Ferrox | optimization, scheduling, routing, allocation, feasibility, solver-backed constraints |

If this app appears to need regression, fuzzy logic, ranking, forecasting, optimization, retrieval, policy, connectors, or adapters, the default answer is to use Mosaic and shape the app request so the specialist can serve it. Local versions are acceptable only as temporary scaffolding with a clear retirement path.

## Controlled Fast Growth

Fast application work is welcome when it strengthens the platform contract.

Allowed temporary scaffolding:

- domain fixtures, seed data, and examples that prove product value
- thin adapters that call the correct lower-layer owner
- test doubles that make a missing lower-layer contract visible
- vertical-specific prompts, labels, and copy that do not pretend to be reusable engines

Not allowed as permanent app code:

- generic ranking or scoring engines when Prism should own the analytic core
- local memory or retrieval systems when Mnemos should own recall
- local policy engines when Arbiter should own decisions
- app-owned provider/tool/vector/storage abstractions when Manifold should own them
- app-owned source connectors when Embassy should own them
- app-owned optimization or feasibility engines when Ferrox should own them
- app-owned convergence or formation loops when Converge and Organism should own them

## Promotion Loop

When the app discovers pressure that feels reusable:

1. Capture the concrete product need in this KB or the app backlog.
2. Classify it as domain-specific, host-level, formation-level, convergence-level, truth-level, or Mosaic-specialist-level.
3. Route it to the right owner instead of deepening app-local machinery.
4. Keep only the smallest app shim needed to keep the product moving.
5. Replace the shim with the platform contract once the lower layer exposes the capability.
6. Record the learning so the next marquee app starts from stronger Bedrock.

## Review Questions

Before building durable app logic, ask:

- Is this domain meaning, or reusable machinery?
- Which lower-layer owner would naturally serve this capability?
- Which Mosaic specialist should be considered before local implementation?
- Is the app producing examples and pressure that improve Bedrock, or hiding a platform gap?
- What would make this scaffold easy to delete after the platform learns it?
