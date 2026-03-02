//! Transitional run-loop entrypoints for the command-side boundary.

use crate::app_state::AppState;

// TODO(v3-viewport-transport): route command-side run-loop ownership through this
// module once the legacy handler shim is retired.
#[allow(dead_code)]
pub(crate) async fn run_loop(app: AppState) {
    crate::handlers::lifecycle::run_loop(app).await;
}
