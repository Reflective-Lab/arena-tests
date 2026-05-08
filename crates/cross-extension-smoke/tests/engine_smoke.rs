//! Smoke test — confirms the engine compiles and converges with platform crates only.
//! Add cross-extension scenarios as new tests in this directory.

use converge_core::{ContextState, Engine};

#[tokio::test]
async fn engine_runs_without_suggestors() {
    let mut engine = Engine::new();
    let result = engine
        .run(ContextState::new())
        .await
        .expect("engine should handle empty suggestor set");
    assert!(result.converged, "empty engine should converge immediately");
}
