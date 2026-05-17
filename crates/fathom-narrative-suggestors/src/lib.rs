//! Converge suggestors and invariants for Fathom — SPARC: propose facts
//! from the EDGAR lakehouse, enforce structural laws on what gets promoted.

pub mod invariants;
pub mod portfolio_seed;
pub mod risk_factor_drift;
pub mod risk_factor_language;

pub use invariants::RiskFactorMassConservationInvariant;
pub use portfolio_seed::PortfolioCoverageSeedSuggestor;
pub use risk_factor_drift::RiskFactorDriftSuggestor;
pub use risk_factor_language::RiskFactorLanguageSuggestor;
