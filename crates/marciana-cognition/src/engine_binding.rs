//! Trusted host binding between a native profile and its engine.

use std::fmt;
use std::sync::Arc;

use grust::SailGraphStore;
use querygraph_memory::cognition::{
    CognitionEngine, CognitionEngineProfile, CognitionError, CognitionOperation, CognitionRequest,
    LiveSailCognitionExecutor, ReferenceCognitionEngine, SailCognitionEngine,
};
use typesec_memory::CognitionProposal;

/// Host-authorized native cognition engine.
///
/// The profile is selected by this closed composition boundary, independently
/// of the public `CognitionEngine` implementation. Arbitrary engines therefore
/// cannot self-report a trusted identity and receive protected memory.
///
/// Custom engines cannot be wrapped through the public API:
///
/// ```compile_fail
/// use std::sync::Arc;
/// use marciana_cognition::CognitionEngineBinding;
/// use querygraph_memory::cognition::CognitionEngine;
///
/// let arbitrary: Arc<dyn CognitionEngine> = unimplemented!();
/// let _ = CognitionEngineBinding::from_engine(arbitrary);
/// ```
#[must_use = "an engine binding must be installed on a cognition application"]
#[derive(Clone)]
pub struct CognitionEngineBinding {
    engine: Arc<dyn CognitionEngine>,
    family: NativeEngineFamily,
}

impl fmt::Debug for CognitionEngineBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CognitionEngineBinding")
            .field("family", &self.family)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
enum NativeEngineFamily {
    Reference,
    Sail,
}

impl CognitionEngineBinding {
    /// Bind Grust's deterministic reference engine.
    pub fn reference() -> Self {
        Self::bind(
            Arc::new(ReferenceCognitionEngine),
            NativeEngineFamily::Reference,
        )
    }

    /// Bind Grust's live Sail engine to an established Spark Connect session.
    pub fn live_sail(store: Arc<SailGraphStore>) -> Self {
        Self::bind(
            Arc::new(SailCognitionEngine::new(LiveSailCognitionExecutor::new(
                store,
            ))),
            NativeEngineFamily::Sail,
        )
    }

    /// Return the fixed native profile for an allowed cognition operation.
    #[must_use]
    pub fn profile(&self, operation: CognitionOperation) -> CognitionEngineProfile {
        match self.family {
            NativeEngineFamily::Reference => CognitionEngineProfile::reference(operation),
            NativeEngineFamily::Sail => CognitionEngineProfile::sail(operation),
        }
    }

    /// Produce an inert proposal using the host-bound engine.
    ///
    /// # Errors
    ///
    /// Returns the fixed, sanitized cognition error supplied by the bound
    /// engine when proposal production fails.
    pub async fn propose(
        &self,
        request: CognitionRequest<'_>,
    ) -> Result<CognitionProposal, CognitionError> {
        self.engine.propose(request).await
    }

    fn bind(engine: Arc<dyn CognitionEngine>, family: NativeEngineFamily) -> Self {
        Self { engine, family }
    }

    /// Bind a test double to the reference profile.
    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn test_reference(engine: Arc<dyn CognitionEngine>) -> Self {
        Self::bind(engine, NativeEngineFamily::Reference)
    }

    /// Bind a test double to the Sail profile.
    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn test_sail(engine: Arc<dyn CognitionEngine>) -> Self {
        Self::bind(engine, NativeEngineFamily::Sail)
    }
}
