use std::sync::Arc;

use querygraph_memory::cognition::{
    CognitionEngine, CognitionEngineProfile, CognitionError, CognitionOperation, CognitionRequest,
};
use typesec_memory::CognitionProposal;

use super::CognitionEngineBinding;

#[test]
fn trusted_host_selects_a_fixed_native_profile_without_invoking_engine() {
    let engine: Arc<dyn CognitionEngine> = Arc::new(UninvokedEngine);
    let reference = CognitionEngineBinding::test_reference(engine.clone());
    let sail = CognitionEngineBinding::test_sail(engine);

    for operation in [
        CognitionOperation::Deduplicate,
        CognitionOperation::Reconcile,
    ] {
        assert_eq!(
            reference.profile(operation),
            CognitionEngineProfile::reference(operation)
        );
        assert_eq!(
            sail.profile(operation),
            CognitionEngineProfile::sail(operation)
        );
        assert_ne!(reference.profile(operation), sail.profile(operation));
    }
}

struct UninvokedEngine;

#[async_trait::async_trait]
impl CognitionEngine for UninvokedEngine {
    async fn propose(
        &self,
        _request: CognitionRequest<'_>,
    ) -> Result<CognitionProposal, CognitionError> {
        unreachable!("profile selection must not invoke the bound engine")
    }
}
