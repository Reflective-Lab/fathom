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
