use std::sync::OnceLock;
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Returns a reference to the global multi-threaded Tokio runtime for Yarac UI background tasks.
pub fn tokio_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("yarac-tokio-worker")
            .build()
            .expect("Failed to initialize Tokio runtime for Yarac UI")
    })
}
