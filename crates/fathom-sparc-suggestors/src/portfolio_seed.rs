//! `PortfolioCoverageSeedSuggestor` — bridges Fathom's analytical layer to
//! the foundation's `PortfolioSuggestor` (and any other knapsack-shaped
//! solvers, e.g. Ferrox `HighsMipSuggestor`).
//!
//! Reads promoted `RiskFactorDrift` and `RiskFactorLanguageDrift` facts from
//! `ContextKey::Proposals`, groups them by CIK, and emits a single
//! `PortfolioRequest` to `ContextKey::Seeds` describing the coverage problem:
//!
//! - **Item per CIK** that has *both* drift signals — gives the solver a
//!   complete picture per candidate.
//! - **Weight = current_count** — each disclosed risk factor is one unit of
//!   analyst-reading capacity. The CLI passes a budget in the same units
//!   (factors-readable-this-week).
//! - **Value = round((1 - jaccard) × 100)** — language churn is the signal
//!   that earns analyst time. Companies that rewrote half their Item 1A get
//!   a value of ~50; companies that didn't change a word get 0.
//!
//! Downstream, `PortfolioSuggestor` (foundation, pure-Rust DP) produces a
//! `PortfolioSelection` to `Strategies`. If `HighsMipSuggestor` (Ferrox,
//! HiGHS MIP) is also registered, both produce competing plans and the
//! engine merges them in deterministic order — the formation pattern from
//! Ferrox's own README.

use async_trait::async_trait;
use converge_optimization::suggestors::portfolio::{PortfolioItem, PortfolioRequest};
use converge_pack::{AgentEffect, Context, ContextKey, ProposedFact, Suggestor};
use fathom_sparc_core::analytic::{RiskFactorDrift, RiskFactorLanguageDrift};
use std::collections::HashMap;

const PROVENANCE: &str = "fathom-sparc:portfolio_seed:v1";
const REQUEST_ID: &str = "fathom-sparc:portfolio:risk-coverage";

pub struct PortfolioCoverageSeedSuggestor;

impl PortfolioCoverageSeedSuggestor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PortfolioCoverageSeedSuggestor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Suggestor for PortfolioCoverageSeedSuggestor {
    fn name(&self) -> &str {
        "portfolio_coverage_seed"
    }

    fn dependencies(&self) -> &[ContextKey] {
        // Reads drift facts from Proposals and the optional CLI-supplied
        // budget directive from Constraints.
        &[ContextKey::Proposals, ContextKey::Constraints]
    }

    fn complexity_hint(&self) -> Option<&'static str> {
        Some("O(n) — n = promoted drift facts; one PortfolioRequest emitted")
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        if already_seeded(ctx) {
            return false;
        }
        let (counts, langs) = parse_drifts(ctx);
        // Need at least one CIK with *both* signals to form a coverage item.
        counts
            .keys()
            .any(|key| langs.contains_key(key))
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let (counts, langs) = parse_drifts(ctx);
        let mut request = build_request(REQUEST_ID, &counts, &langs);
        if request.items.is_empty() {
            return AgentEffect::empty();
        }
        // If the CLI seeded a budget directive into Constraints, honour it;
        // otherwise build_request's "fits everything" default applies.
        if let Some(budget) = read_budget_directive(ctx) {
            request.budget = budget;
        }
        let content = match serde_json::to_string(&request) {
            Ok(s) => s,
            Err(_) => return AgentEffect::empty(),
        };
        let proposal = ProposedFact::new(
            ContextKey::Seeds,
            format!("portfolio-request:{}", request.id),
            content,
            PROVENANCE,
        );
        AgentEffect::with_proposal(proposal)
    }
}

const BUDGET_DIRECTIVE_ID: &str = "portfolio-budget:risk-coverage";

fn read_budget_directive(ctx: &dyn Context) -> Option<i64> {
    ctx.get(ContextKey::Constraints)
        .iter()
        .find(|f| f.id().as_str() == BUDGET_DIRECTIVE_ID)
        .and_then(|f| f.content().trim().parse::<i64>().ok())
}

fn already_seeded(ctx: &dyn Context) -> bool {
    let seed_id = format!("portfolio-request:{REQUEST_ID}");
    ctx.get(ContextKey::Seeds)
        .iter()
        .any(|f| f.id().as_str() == seed_id)
}

type DriftMap<T> = HashMap<(String, u16), T>;

fn parse_drifts(ctx: &dyn Context) -> (DriftMap<RiskFactorDrift>, DriftMap<RiskFactorLanguageDrift>) {
    let mut counts = HashMap::new();
    let mut langs = HashMap::new();
    for fact in ctx.get(ContextKey::Proposals) {
        if let Ok(d) = serde_json::from_str::<RiskFactorDrift>(fact.content()) {
            counts.insert((d.current.cik.as_str().to_string(), d.current.fiscal_year), d);
            continue;
        }
        if let Ok(d) = serde_json::from_str::<RiskFactorLanguageDrift>(fact.content()) {
            langs.insert((d.current.cik.as_str().to_string(), d.current.fiscal_year), d);
        }
    }
    (counts, langs)
}

/// Pure helper — given the parsed drifts, produce the `PortfolioRequest`.
/// Exposed for direct testing.
pub fn build_request(
    id: &str,
    counts: &DriftMap<RiskFactorDrift>,
    langs: &DriftMap<RiskFactorLanguageDrift>,
) -> PortfolioRequest {
    let mut items = Vec::new();
    let mut total_weight: i64 = 0;
    // Deterministic order: sort by CIK then year so the request is stable
    // across runs (and integration tests).
    let mut keys: Vec<&(String, u16)> = counts.keys().collect();
    keys.sort();
    for key in keys {
        let count = counts.get(key).expect("just iterated");
        let Some(lang) = langs.get(key) else { continue };
        let weight = count.current_count as i64;
        let value = ((1.0 - lang.jaccard_similarity) * 100.0).round() as i64;
        let label = format!("{}::FY{}", key.0, key.1);
        total_weight = total_weight.saturating_add(weight);
        items.push(PortfolioItem {
            label,
            weight,
            value,
        });
    }
    PortfolioRequest {
        id: id.to_string(),
        items,
        budget: total_weight, // placeholder — CLI overrides
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fathom_sparc_core::{Cik, FilingId, FormType};

    fn filing(cik: &str, fy: u16) -> FilingId {
        FilingId {
            cik: Cik::new(cik),
            form: FormType::TenK,
            fiscal_year: fy,
        }
    }

    fn count(cik: &str, fy: u16, current_count: usize, delta: i32) -> RiskFactorDrift {
        let prior_count = (current_count as i32 - delta).max(0) as usize;
        RiskFactorDrift {
            current: filing(cik, fy),
            prior: filing(cik, fy - 1),
            current_count,
            prior_count,
            delta,
        }
    }

    fn lang(cik: &str, fy: u16, jaccard: f64) -> RiskFactorLanguageDrift {
        RiskFactorLanguageDrift {
            current: filing(cik, fy),
            prior: filing(cik, fy - 1),
            identical_count: 20,
            jaccard_similarity: jaccard,
            added: vec!["a".into()],
            removed: vec!["r".into()],
        }
    }

    #[test]
    fn builds_one_item_per_paired_cik() {
        let mut counts = DriftMap::new();
        let mut langs = DriftMap::new();
        // 3 CIKs all with both signals.
        for (cik, fy, c, j) in [
            ("0000320193", 2025, 27, 0.618),
            ("0000789019", 2025, 20, 0.55),
            ("0001045810", 2026, 23, 0.80),
        ] {
            counts.insert((cik.to_string(), fy), count(cik, fy, c, -1));
            langs.insert((cik.to_string(), fy), lang(cik, fy, j));
        }
        let req = build_request("test", &counts, &langs);
        assert_eq!(req.items.len(), 3);
        // Deterministic order — sorted by CIK then year.
        assert_eq!(req.items[0].label, "0000320193::FY2025");
        assert_eq!(req.items[1].label, "0000789019::FY2025");
        assert_eq!(req.items[2].label, "0001045810::FY2026");
        // Apple weight = current_count = 27.
        assert_eq!(req.items[0].weight, 27);
        // Apple value = round((1 - 0.618) * 100) = 38.
        assert_eq!(req.items[0].value, 38);
        // Default budget = sum of weights (27 + 20 + 23 = 70).
        assert_eq!(req.budget, 70);
    }

    #[test]
    fn skips_ciks_missing_either_signal() {
        let mut counts = DriftMap::new();
        let mut langs = DriftMap::new();
        counts.insert(
            ("0000000001".to_string(), 2025),
            count("0000000001", 2025, 10, 0),
        );
        // Different CIK has language drift but no count drift — filtered out.
        langs.insert(
            ("0000000002".to_string(), 2025),
            lang("0000000002", 2025, 0.5),
        );
        let req = build_request("test", &counts, &langs);
        assert!(req.items.is_empty());
    }

    #[test]
    fn higher_churn_higher_value() {
        let mut counts = DriftMap::new();
        let mut langs = DriftMap::new();
        // A: jaccard 0.9 → low churn → value ~10.
        // B: jaccard 0.5 → high churn → value 50.
        for (cik, j) in [("A", 0.9), ("B", 0.5)] {
            counts.insert((cik.to_string(), 2025), count(cik, 2025, 20, -1));
            langs.insert((cik.to_string(), 2025), lang(cik, 2025, j));
        }
        let req = build_request("t", &counts, &langs);
        let a = req.items.iter().find(|i| i.label.starts_with("A::")).unwrap();
        let b = req.items.iter().find(|i| i.label.starts_with("B::")).unwrap();
        assert!(b.value > a.value, "B (churn) should outvalue A (stable)");
    }

    #[test]
    fn suggestor_metadata() {
        let s = PortfolioCoverageSeedSuggestor::new();
        assert_eq!(s.name(), "portfolio_coverage_seed");
        assert_eq!(
            s.dependencies(),
            &[ContextKey::Proposals, ContextKey::Constraints]
        );
    }
}
