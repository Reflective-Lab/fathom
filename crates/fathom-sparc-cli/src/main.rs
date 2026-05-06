//! Fathom — SPARC CLI.
//!
//! `fathom-sparc analyse <CIK>` discovers JSON fixtures under `fixtures/`
//! for that CIK, seeds them as inputs into a `ContextState`, registers the
//! risk-factor drift and language suggestors with a Converge `Engine`, runs
//! the convergence loop, and prints the promoted facts (with provenance) as
//! JSON.
//!
//! The engine is the load-bearing piece: it owns eligibility scheduling,
//! deterministic merge order, the promotion gate that turns `ProposedFact`
//! into authoritative `Fact`, and the integrity proof for the final
//! context. None of that is fake — the same engine drives Converge
//! consumers like Organism and Wolfgang.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, bail};
use clap::{Parser, Subcommand};
use converge_kernel::{
    ContextState, Engine, EngineHitlPolicy, GateDecision, RunResult, TimeoutPolicy,
};
use converge_pack::{ContextKey, Fact};
use fathom_sparc_core::{Cik, RiskFactorSection};
use fathom_sparc_ingest::load_risk_factor_fixture;
use fathom_sparc_suggestors::{
    RiskFactorDriftSuggestor, RiskFactorLanguageSuggestor, RiskFactorMassConservationInvariant,
};

/// Confidence floor — proposals at or below this trigger a HITL pause.
/// `RiskFactorLanguageSuggestor` sets confidence = Jaccard similarity, so
/// any consecutive-year pair with substantial language churn (Jaccard ≤ 0.7)
/// requires explicit approval before promotion.
const HITL_CONFIDENCE_THRESHOLD: f64 = 0.7;

const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures");

#[derive(Parser)]
#[command(
    name = "fathom-sparc",
    about = "Convergence-driven analysis of public-company financial filings"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Materialise a HuggingFace dataset slice into the local Iceberg lakehouse.
    /// Not yet implemented; fixtures under `fixtures/` are the on-ramp.
    Ingest,
    /// Run the engine for `cik`: register both suggestors, converge, print
    /// promoted facts as JSON.
    Analyse {
        /// SEC CIK (Central Index Key), e.g. 0000320193 for Apple.
        cik: String,
        /// Override the fixtures directory.
        #[arg(long, default_value = FIXTURES_DIR)]
        fixtures: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match Cli::parse().command {
        Command::Ingest => {
            tracing::info!("ingest pipeline not yet implemented; use fixtures under fixtures/");
            Ok(())
        }
        Command::Analyse { cik, fixtures } => analyse(Cik::new(cik), &fixtures).await,
    }
}

async fn analyse(cik: Cik, fixtures_dir: &std::path::Path) -> anyhow::Result<()> {
    let sections = load_sections_for_cik(&cik, fixtures_dir)?;
    if sections.len() < 2 {
        bail!(
            "found {} fixture(s) for CIK {} in {}; need at least 2 to compute drift",
            sections.len(),
            cik.as_str(),
            fixtures_dir.display()
        );
    }
    tracing::info!(
        cik = cik.as_str(),
        count = sections.len(),
        "loaded fixtures"
    );

    let context = seed_context(&sections)?;
    let mut engine = Engine::new();
    engine.register_suggestor(RiskFactorDriftSuggestor::new());
    engine.register_suggestor(RiskFactorLanguageSuggestor::new());
    engine.register_invariant(RiskFactorMassConservationInvariant);
    engine.set_hitl_policy(EngineHitlPolicy {
        confidence_threshold: Some(HITL_CONFIDENCE_THRESHOLD),
        gated_keys: Vec::new(),
        timeout: TimeoutPolicy::default(),
    });

    let mut gated: Vec<String> = Vec::new();
    let mut step = engine.run_with_hitl(context).await;
    let result = loop {
        match step {
            RunResult::Complete(r) => {
                break r.map_err(|e| anyhow::anyhow!("engine run failed: {e:?}"))?;
            }
            RunResult::HitlPause(pause) => {
                let summary = pause.request.summary.clone();
                let gate_id = pause.request.gate_id.clone();
                gated.push(format!(
                    "gate={gate_id} cycle={cycle} summary={summary:?}",
                    cycle = pause.cycle
                ));
                tracing::warn!(
                    %gate_id,
                    cycle = pause.cycle,
                    summary = %summary,
                    "auto-approving HITL gate (confidence ≤ {HITL_CONFIDENCE_THRESHOLD}); \
                     interactive review path lands when there's a UI to host it"
                );
                let decision = GateDecision::approve(gate_id, "fathom-sparc:auto-approver");
                step = engine.resume(*pause, decision).await;
            }
        }
    };

    tracing::info!(
        cycles = result.cycles,
        converged = result.converged,
        stop_reason = ?result.stop_reason,
        gated = gated.len(),
        "engine finished"
    );
    if !gated.is_empty() {
        eprintln!(
            "INFO: {} HITL gate(s) auto-approved during this run:",
            gated.len()
        );
        for g in &gated {
            eprintln!("  - {g}");
        }
    }

    let promoted = result.context.get(ContextKey::Proposals);
    if promoted.is_empty() {
        println!("no proposals promoted to facts");
        return Ok(());
    }

    let view: Vec<FactView<'_>> = promoted.iter().map(FactView::from).collect();
    println!("{}", serde_json::to_string_pretty(&view)?);
    Ok(())
}

fn load_sections_for_cik(
    cik: &Cik,
    fixtures_dir: &std::path::Path,
) -> anyhow::Result<Vec<RiskFactorSection>> {
    let mut out = Vec::new();
    let entries = fs::read_dir(fixtures_dir)
        .with_context(|| format!("reading fixtures dir {}", fixtures_dir.display()))?;
    for entry in entries {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match load_risk_factor_fixture(&path) {
            Ok(s) if &s.filing.cik == cik => out.push(s),
            Ok(_) => {}
            Err(err) => tracing::warn!(?path, error = %err, "skipping unreadable fixture"),
        }
    }
    out.sort_by_key(|s| s.filing.fiscal_year);
    Ok(out)
}

/// Stages each loaded section as an *input* into a fresh `ContextState`.
/// The engine drains these proposals at cycle 0 and promotes them to
/// authoritative `Fact`s under `ContextKey::Signals` — which is exactly
/// what the suggestors then read from.
fn seed_context(sections: &[RiskFactorSection]) -> anyhow::Result<ContextState> {
    let mut ctx = ContextState::new();
    for s in sections {
        let id = format!(
            "filing::{}::{}",
            s.filing.cik.as_str(),
            s.filing.fiscal_year
        );
        let content = serde_json::to_string(s)?;
        ctx.add_input_with_provenance(
            ContextKey::Signals,
            id.clone(),
            content,
            "fathom-sparc:fixture",
        )
        .map_err(|e| anyhow::anyhow!("add_input failed for {id}: {e:?}"))?;
    }
    Ok(ctx)
}

/// Display-friendly projection of a promoted `Fact` — strips the internal id
/// type wrapper so the JSON output reads cleanly. Provenance is recovered
/// from the fact's promotion record (the actor that promoted it).
#[derive(serde::Serialize)]
struct FactView<'a> {
    key: &'a str,
    id: String,
    content: serde_json::Value,
    promoted_by: String,
}

impl<'a> From<&'a Fact> for FactView<'a> {
    fn from(f: &'a Fact) -> Self {
        let key = match f.key() {
            ContextKey::Proposals => "Proposals",
            ContextKey::Signals => "Signals",
            ContextKey::Hypotheses => "Hypotheses",
            ContextKey::Strategies => "Strategies",
            ContextKey::Constraints => "Constraints",
            ContextKey::Seeds => "Seeds",
            ContextKey::Competitors => "Competitors",
            ContextKey::Evaluations => "Evaluations",
            ContextKey::Diagnostic => "Diagnostic",
            ContextKey::Votes => "Votes",
            ContextKey::Disagreements => "Disagreements",
            ContextKey::ConsensusOutcomes => "ConsensusOutcomes",
        };
        let content = serde_json::from_str(&f.content)
            .unwrap_or(serde_json::Value::String(f.content.clone()));
        Self {
            key,
            id: f.id.to_string(),
            content,
            promoted_by: format!("{:?}", f.promotion_record().approver()),
        }
    }
}
