//! The one sanctioned sync→async bridge (FABLE-MEMORY-1 §5.1).
//!
//! `MemoryStore` is synchronous by design; Grust's `GraphStore` is async.
//! The bridge owns a dedicated current-thread runtime, driven directly when
//! no runtime is on the calling thread and from a scoped thread when one is
//! (so calling the vault from inside tokio — e.g. an MCP server — cannot
//! panic).

use std::future::Future;

/// A dedicated current-thread runtime, safe to call from inside or outside
/// another tokio runtime.
pub(crate) struct Bridge {
    rt: Option<tokio::runtime::Runtime>,
}

impl Bridge {
    pub(crate) fn new() -> Self {
        Self {
            rt: Some(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("bridge runtime builds"),
            ),
        }
    }

    pub(crate) fn run<T: Send>(&self, fut: impl Future<Output = T> + Send) -> T {
        let rt = self.rt.as_ref().expect("bridge runtime is available");
        if tokio::runtime::Handle::try_current().is_ok() {
            // Already inside a runtime: block_on here would panic. Drive the
            // bridge runtime from a scoped thread instead.
            std::thread::scope(|scope| {
                scope
                    .spawn(|| rt.block_on(fut))
                    .join()
                    .expect("bridge thread completes")
            })
        } else {
            rt.block_on(fut)
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        let Some(rt) = self.rt.take() else {
            return;
        };
        if tokio::runtime::Handle::try_current().is_ok() {
            // Dropping a runtime may block while its workers shut down, which
            // Tokio rejects inside an async context. Finish shutdown on a
            // plain thread so a TursoMemoryStore can be owned directly by an
            // async service without special drop choreography.
            std::thread::spawn(move || drop(rt))
                .join()
                .expect("bridge runtime shutdown completes");
        } else {
            drop(rt);
        }
    }
}
