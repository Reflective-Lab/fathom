//! Custom Converge invariants for Fathom — SPARC.
//!
//! Invariants are the engine's "law" — checked at well-defined points in the
//! convergence loop, with violations halting promotion or rejecting results.
//! The invariant here enforces a mathematical identity that *must* hold
//! between the count-drift and language-drift facts when both are emitted
//! for the same `(CIK, fiscal_year)` pair. If it doesn't, something
//! upstream (parser, fixture, suggestor) is wrong.

use std::collections::HashMap;

use converge_core::invariant::Violation;
use converge_pack::{Context, ContextKey};
use converge_kernel::{Invariant, InvariantClass, InvariantResult};
use fathom_sparc_core::analytic::{RiskFactorDrift, RiskFactorLanguageDrift};

/// Mass conservation: for any `(CIK, fiscal_year)` where both a count drift
/// and a language drift were promoted, the identity
/// `added.len() - removed.len() == count.delta` must hold.
///
/// **Why this is a real invariant, not just a heuristic.** The count delta
/// *is* the number of headings added minus the number removed — by
/// definition. If the language suggestor's `added`/`removed` lists don't
/// agree with the count suggestor's `delta`, the two suggestors are reading
/// different inputs, the parser broke, or one of them has a bug. None of
/// those is an analytical disagreement worth surfacing — it's a structural
/// inconsistency that should fail the run.
///
/// Class is `Acceptance` — checked once at convergence claim. Violations
/// reject the run rather than gating it for human review.
pub struct RiskFactorMassConservationInvariant;

impl Invariant for RiskFactorMassConservationInvariant {
    fn name(&self) -> &str {
        "risk_factor_mass_conservation"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Acceptance
    }

    fn check(&self, ctx: &dyn Context) -> InvariantResult {
        let mut counts: HashMap<(String, u16), RiskFactorDrift> = HashMap::new();
        let mut langs: HashMap<(String, u16), RiskFactorLanguageDrift> = HashMap::new();

        for fact in ctx.get(ContextKey::Proposals) {
            if let Ok(d) = serde_json::from_str::<RiskFactorDrift>(&fact.content) {
                counts.insert((d.current.cik.as_str().to_string(), d.current.fiscal_year), d);
                continue;
            }
            if let Ok(d) = serde_json::from_str::<RiskFactorLanguageDrift>(&fact.content) {
                langs.insert((d.current.cik.as_str().to_string(), d.current.fiscal_year), d);
            }
        }

        for (key, count) in &counts {
            let Some(lang) = langs.get(key) else { continue };
            let lang_delta = lang.added.len() as i32 - lang.removed.len() as i32;
            if lang_delta != count.delta {
                return InvariantResult::Violated(Violation::new(format!(
                    "mass conservation violated for CIK {} FY{}: \
                     count delta = {}, language (added - removed) = ({} - {}) = {}",
                    count.current.cik.as_str(),
                    count.current.fiscal_year,
                    count.delta,
                    lang.added.len(),
                    lang.removed.len(),
                    lang_delta,
                )));
            }
        }

        InvariantResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use converge_pack::Fact;
    use converge_pack::fact::kernel_authority::new_fact;
    use fathom_sparc_core::{Cik, FilingId, FormType};

    struct MockCtx {
        proposals: Vec<Fact>,
    }
    impl Context for MockCtx {
        fn has(&self, k: ContextKey) -> bool {
            matches!(k, ContextKey::Proposals) && !self.proposals.is_empty()
        }
        fn get(&self, k: ContextKey) -> &[Fact] {
            match k {
                ContextKey::Proposals => &self.proposals,
                _ => &[],
            }
        }
    }

    fn filing(cik: &str, fy: u16) -> FilingId {
        FilingId {
            cik: Cik::new(cik),
            form: FormType::TenK,
            fiscal_year: fy,
        }
    }

    fn count_fact(cik: &str, fy: u16, prior_count: usize, current_count: usize) -> Fact {
        let drift = RiskFactorDrift {
            current: filing(cik, fy),
            prior: filing(cik, fy - 1),
            current_count,
            prior_count,
            delta: current_count as i32 - prior_count as i32,
        };
        new_fact(
            ContextKey::Proposals,
            format!("risk_factor_drift::{cik}::{fy}"),
            serde_json::to_string(&drift).unwrap(),
        )
    }

    fn lang_fact(cik: &str, fy: u16, added: usize, removed: usize) -> Fact {
        let drift = RiskFactorLanguageDrift {
            current: filing(cik, fy),
            prior: filing(cik, fy - 1),
            identical_count: 20,
            jaccard_similarity: 0.8,
            added: (0..added).map(|i| format!("a-{i}")).collect(),
            removed: (0..removed).map(|i| format!("r-{i}")).collect(),
        };
        new_fact(
            ContextKey::Proposals,
            format!("risk_factor_language::{cik}::{fy}"),
            serde_json::to_string(&drift).unwrap(),
        )
    }

    #[test]
    fn passes_when_identity_holds() {
        // delta = -1 → added = 6, removed = 7, diff = -1 ✓
        let ctx = MockCtx {
            proposals: vec![
                count_fact("0000320193", 2025, 28, 27),
                lang_fact("0000320193", 2025, 6, 7),
            ],
        };
        assert!(RiskFactorMassConservationInvariant.check(&ctx).is_ok());
    }

    #[test]
    fn passes_when_only_count_present() {
        let ctx = MockCtx {
            proposals: vec![count_fact("0000320193", 2025, 28, 27)],
        };
        assert!(RiskFactorMassConservationInvariant.check(&ctx).is_ok());
    }

    #[test]
    fn violates_when_identity_breaks() {
        // delta = -1 but language reports added=6, removed=8 → diff = -2 ≠ -1
        let ctx = MockCtx {
            proposals: vec![
                count_fact("0000320193", 2025, 28, 27),
                lang_fact("0000320193", 2025, 6, 8),
            ],
        };
        let result = RiskFactorMassConservationInvariant.check(&ctx);
        assert!(result.is_violated());
        if let InvariantResult::Violated(v) = result {
            assert!(v.reason.contains("mass conservation"));
            assert!(v.reason.contains("0000320193"));
        }
    }

    #[test]
    fn violates_when_count_grows_but_language_does_not() {
        let ctx = MockCtx {
            proposals: vec![
                count_fact("0000789019", 2025, 30, 35), // delta = +5
                lang_fact("0000789019", 2025, 2, 0),    // diff = +2
            ],
        };
        assert!(RiskFactorMassConservationInvariant
            .check(&ctx)
            .is_violated());
    }
}
