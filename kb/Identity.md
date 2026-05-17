# Fathom — what the app is

**Status:** post-rename (2026-05-17). The app moved from `fathom-sparc`
to `fathom-narrative` after the SEC-specific transport was lifted into
the `embassy-sec-edgar` port.

## The product

Fathom is a temporal-narrative analyzer for corporate disclosures.
Specifically, it watches *how the language of risk changes* across:

1. A single company's filings over time (year-over-year drift)
2. A cohort of companies at the same point in time (portfolio-level patterns)

The differentiator is the **delta**, not the snapshot. Reading any single
10-K tells you what a company is currently worried about. Reading five
consecutive 10-Ks tells you what *changed*. Reading the 10-Ks of fifteen
peer companies in the same quarter tells you whether the change is a
company-specific concern or a sector-wide narrative shift.

That's the signal Fathom surfaces.

## The four pillars

Fathom's analytical substance lives in `fathom-narrative-suggestors/`:

### 1. `risk_factor_drift`

Year-over-year topical drift. Which risks are new this year, which dropped
out, which got reframed under different headings. Computes Jaccard
similarity on the risk-factor heading set, but the Jaccard number isn't
the product — the *which-ones-are-new* and *which-ones-dropped* lists are
the product.

### 2. `risk_factor_language`

Linguistic features on the risk text itself: hedging density, certainty
modals, jargon load, sentence complexity, sentiment of the surrounding
sentences. The signal is whether the company is getting more cautious,
more confident, or more obfuscatory in *how* it talks about risk —
independently of *what* it's talking about.

This is the pillar that defends against the easy gaming of pillar 1.
("We didn't drop any risk factors, but we made the disclosures so vague
they say nothing" is a real pattern; only language analysis catches it.)

### 3. `portfolio_seed`

Cross-sectional aggregation. If seven of twelve portfolio companies
suddenly add the same new risk factor this quarter, that's a signal
nothing about reading one filing would catch. The portfolio is the
unit of analysis, not the filing.

### 4. `invariants`

Verifier rules that say "if drift signal X fires, language signal Y must
agree." This is the policy layer that makes Fathom's outputs auditable
rather than just clever. Two suggestors disagreeing on the same underlying
filing is a flag for human review, not a failure mode.

## Where SEC lives now

It doesn't live in this app. The SEC contract — User-Agent, rate-limit
politeness, the three observed 10-K markup-selector patterns, the Item-N
section locator heuristic — moved to `embassy-sec-edgar` in
mosaic-extensions. The app pulls SEC filings *through* that port and
focuses entirely on what to do with the bytes once they arrive.

This was the rename's load-bearing change. SEC was never Fathom's
differentiator; SEC was the prerequisite to do the actual work. Lifting
it out makes the actual work visible and lets the same analytical engine
point at any future narrative corpus (Bolagsverket annual reports, EU
prospectuses, arXiv research-program statements) without changing the
app — just by swapping the embassy port.

## Why "narrative", not "filings"

The corpus-agnostic framing matters because the same drift / language /
portfolio-aggregation machinery works on any time-indexed narrative
corpus. Today the production corpus is SEC 10-Ks. Tomorrow's plausible
adjacents:

- Swedish annual reports via `embassy-bolagsverket` (when that port grows
  a `live` feature)
- UK Annual Reports via `embassy-companies-house`
- EU prospectuses (no port yet)
- Earnings-call transcripts (no port yet)
- arXiv abstracts for tracking research program shifts (a stretch but
  the same shape of work)

Each new corpus is a new embassy port. Fathom's suggestors barely change.

## What got renamed in this change

| Old | New |
|-----|-----|
| `fathom-sparc/` workspace dir | `fathom-narrative/` |
| `fathom-sparc-core` crate | `fathom-narrative-core` |
| `fathom-sparc-ingest` crate | `fathom-narrative-ingest` *(rename to `-synthesis` is planned when the embassy-sec-edgar cutover lands)* |
| `fathom-sparc-suggestors` crate | `fathom-narrative-suggestors` |
| `fathom-sparc-cli` crate | `fathom-narrative-cli` |
| `policies/fathom_sparc.cedar` | `policies/fathom_narrative.cedar` |
| `fathom_sparc_core::*` imports | `fathom_narrative_core::*` |
| GitHub repo `Reflective-Lab/fathom-sparc` | TODO — needs `gh repo rename` |

The Tauri desktop app at `apps/desktop/src-tauri/` keeps its scaffold;
only its dependency identifiers changed.

## SPARC — what it was, why it dropped

`SPARC` plausibly read as "SEC-Portfolio-Analysis-Risk-Comparison" or
similar. The "SEC" piece was load-bearing in the name as long as the
app owned the SEC ingest. Once the ingest moved to embassy and the app's
analytical work proved corpus-agnostic, the acronym lost its grounding.

Naming options considered during the rename:
- Keep `fathom-sparc`, redefine SPARC as "Signal-Pattern Analysis on
  Risk Corpora" — keeps brand equity but the renamed acronym felt
  retrofitted
- Rename to `fathom-drift` — sharper but loses the language-analysis
  pillar
- Rename to `fathom-filings` — corpus-typed, restricts to one corpus
- Rename to `fathom-narrative` — **chosen.** Corpus-agnostic, captures
  the temporal-language angle, fits the workspace alias-purpose pattern,
  and is durable across future corpora without rebranding
