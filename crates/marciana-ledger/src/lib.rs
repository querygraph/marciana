//! Canonical assertion identity and lifecycle rules for Marciana's ledger.
//!
//! This crate deliberately contains no store or authorization handle. Durable
//! adapters persist these values through the TypeSec-authorized, guarded
//! mutation path; they do not reinterpret temporal or transition semantics.

mod assertion;
mod error;
mod migration;
mod query;
mod temporal;
mod transition;

pub use assertion::{Assertion, AssertionId, AssertionLineage, Confidence};
pub use error::LedgerError;
pub use migration::LegacyRelation;
pub use query::AssertionQuery;
pub use temporal::TemporalInterval;
pub use transition::{AssertionState, AssertionTransition, TransitionEvidence};
