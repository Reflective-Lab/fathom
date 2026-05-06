//! End-to-end smoke test: run the binary against real Apple fixtures and
//! assert that the Converge engine promoted both suggestors' proposals into
//! authoritative facts with the expected shape.
//!
//! Cargo provides `CARGO_BIN_EXE_<name>` to integration tests so we can spawn
//! the just-built binary without hard-coding a path.

use std::process::Command;

#[test]
fn engine_promotes_both_drift_facts_for_apple_fy24_to_fy25() {
    let output = Command::new(env!("CARGO_BIN_EXE_fathom-sparc"))
        .args(["analyse", "0000320193"])
        .output()
        .expect("spawn fathom binary");

    assert!(
        output.status.success(),
        "binary exited non-zero: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // The language suggestor's confidence == Jaccard similarity. Apple's
    // FY24→FY25 language drift has Jaccard ≈ 0.618, well below the 0.7
    // HITL threshold the CLI configures, so exactly one gate must fire and
    // be auto-approved before the engine completes.
    assert!(
        stderr.contains("HITL gate(s) auto-approved"),
        "expected HITL pause + auto-approve in stderr; got: {stderr}"
    );
    assert!(
        stderr.contains("risk_factor_language::0000320193::2025"),
        "HITL gate should reference the language drift proposal; got: {stderr}"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let facts: Vec<serde_json::Value> =
        serde_json::from_str(&stdout).expect("stdout is JSON array");
    assert_eq!(facts.len(), 2, "expected two promoted facts");

    // Every fact must carry the engine as the promoting actor — that's the
    // real proof the convergence loop ran (and that the invariant accepted
    // the result), rather than a hand-rolled context.
    for f in &facts {
        let promoted_by = f["promoted_by"].as_str().expect("promoted_by");
        assert!(
            promoted_by.contains("converge-engine"),
            "fact not promoted by engine: {promoted_by}"
        );
        assert_eq!(f["key"], "Proposals");
    }

    // Suggestor identity is encoded in the fact id (the engine doesn't
    // re-expose the ProposedFact's free-form provenance as a top-level field).
    let by_suggestor: std::collections::HashMap<&str, &serde_json::Value> = facts
        .iter()
        .map(|f| {
            let id = f["id"].as_str().unwrap();
            let prefix = id.split("::").next().unwrap();
            (prefix, f)
        })
        .collect();

    let count = by_suggestor
        .get("risk_factor_drift")
        .expect("count drift fact");
    assert_eq!(count["content"]["current"]["fiscal_year"], 2025);
    assert_eq!(count["content"]["prior"]["fiscal_year"], 2024);
    assert_eq!(count["content"]["current_count"], 27);
    assert_eq!(count["content"]["prior_count"], 28);
    assert_eq!(count["content"]["delta"], -1);

    let language = by_suggestor
        .get("risk_factor_language")
        .expect("language drift fact");
    let content = &language["content"];
    assert_eq!(content["current"]["fiscal_year"], 2025);
    assert_eq!(content["prior"]["fiscal_year"], 2024);
    let identical = content["identical_count"].as_u64().expect("identical_count");
    let added = content["added"].as_array().expect("added array");
    let removed = content["removed"].as_array().expect("removed array");
    let jaccard = content["jaccard_similarity"].as_f64().expect("jaccard f64");
    assert!(
        (18..=25).contains(&identical),
        "identical={identical} outside expected window"
    );
    assert!(
        !added.is_empty() && !removed.is_empty(),
        "expected non-trivial added+removed lists; added={} removed={}",
        added.len(),
        removed.len()
    );
    assert!(
        jaccard > 0.4 && jaccard < 1.0,
        "jaccard={jaccard} outside expected window"
    );
}

#[test]
fn analyse_unknown_cik_fails_with_clear_message() {
    let output = Command::new(env!("CARGO_BIN_EXE_fathom-sparc"))
        .args(["analyse", "9999999999"])
        .output()
        .expect("spawn fathom binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("9999999999") && stderr.contains("need at least 2"),
        "expected helpful error, got: {stderr}"
    );
}

#[test]
fn portfolio_tight_budget_picks_highest_value_cik() {
    // With budget=23 only MSFT (weight 20) fits; Apple (27) and NVDA (23)
    // can't both fit alongside MSFT, but MSFT alone is the cheapest entry
    // with the highest signal value (highest language churn in this batch).
    let output = Command::new(env!("CARGO_BIN_EXE_fathom-sparc"))
        .args(["portfolio", "--budget=23"])
        .output()
        .expect("spawn fathom binary");
    assert!(
        output.status.success(),
        "binary exited non-zero: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout is JSON envelope");
    let selections = envelope["portfolio_selections"]
        .as_array()
        .expect("portfolio_selections array");
    assert_eq!(selections.len(), 1, "exactly one foundation selection expected");
    let sel = &selections[0];
    let selected = sel["selected"].as_array().expect("selected array");
    assert!(
        !selected.is_empty(),
        "tight budget should still select at least one CIK"
    );
    assert!(
        sel["total_weight"].as_i64().unwrap() <= 23,
        "selected weight must respect budget"
    );
}

#[test]
fn portfolio_wide_budget_includes_all_ciks() {
    // Sum of weights for the three real fixtures is 70; budget 100 should
    // select everything with utilization == 1.0 (relative to actual weight,
    // not the budget — the foundation suggestor reports total_weight/budget).
    let output = Command::new(env!("CARGO_BIN_EXE_fathom-sparc"))
        .args(["portfolio", "--budget=100"])
        .output()
        .expect("spawn fathom binary");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout is JSON envelope");
    let selections = envelope["portfolio_selections"]
        .as_array()
        .expect("portfolio_selections array");
    let selected = selections[0]["selected"].as_array().unwrap();
    assert_eq!(
        selected.len(),
        3,
        "with a generous budget the solver should pick all 3 candidate CIKs; got {selected:?}"
    );
}

/// Only runs with `--features=ferrox-mip`. Asserts that the Ferrox HiGHS
/// MIP solver registers, runs, and produces a `MipPlan` whose objective
/// agrees with the foundation `PortfolioSuggestor`'s selection — the
/// canonical multi-solver pattern: two algorithms, same problem, same
/// answer (because both are provably optimal).
#[cfg(feature = "ferrox-mip")]
#[test]
fn portfolio_mip_solver_agrees_with_foundation() {
    let output = Command::new(env!("CARGO_BIN_EXE_fathom-sparc"))
        .args(["portfolio", "--budget=50"])
        .output()
        .expect("spawn fathom binary");
    assert!(
        output.status.success(),
        "binary exited non-zero: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout is JSON envelope");

    let foundation = envelope["portfolio_selections"]
        .as_array()
        .expect("portfolio_selections array");
    let mip = envelope["mip_plans"].as_array().expect("mip_plans array");
    assert_eq!(foundation.len(), 1, "expected one foundation selection");
    assert_eq!(mip.len(), 1, "expected one Ferrox MIP plan");

    let foundation_value = foundation[0]["total_value"]
        .as_i64()
        .expect("foundation total_value");
    let mip_objective = mip[0]["objective_value"]
        .as_f64()
        .expect("mip objective_value");
    assert_eq!(
        foundation_value as f64, mip_objective,
        "foundation DP and Ferrox HiGHS MIP must agree on optimal value"
    );

    let mip_status = mip[0]["status"].as_str().expect("mip status");
    assert_eq!(mip_status, "optimal", "HiGHS should prove optimality");
    let mip_gap = mip[0]["mip_gap"].as_f64().expect("mip_gap");
    assert_eq!(mip_gap, 0.0, "optimal MIP gap is zero");
}
