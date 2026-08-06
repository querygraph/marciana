//! Bounded convenience renderers for an already authorized context bundle.

use std::fmt::Write;

use crate::context::ContextBundle;

/// Maximum rendered bundle size.
pub const MAX_CONTEXT_RENDER_BYTES: usize = 4 * 1024 * 1024;

/// Rendering failure that does not echo protected input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ContextRenderError {
    #[error("context rendering exceeded its fixed output bound")]
    OutputTooLarge,
    #[error("context rendering failed")]
    Formatting,
}

/// Render visible memories with stable citation IDs; redacted items are metadata-only.
///
/// # Errors
///
/// Returns [`ContextRenderError`] when the bundle violates the bounded
/// rendering contract.
pub fn render_text(bundle: &ContextBundle) -> Result<String, ContextRenderError> {
    let mut output = format!(
        "plan={}\nas_of={}\nreceipt={}\n",
        bundle.plan_digest, bundle.as_of, bundle.receipt_digest
    );
    for memory in &bundle.memories {
        writeln!(&mut output, "[{}] {}", memory.id, memory.content.text)
            .map_err(|_| ContextRenderError::Formatting)?;
    }
    for redacted in &bundle.redacted {
        writeln!(&mut output, "[{}] <redacted>", redacted.id)
            .map_err(|_| ContextRenderError::Formatting)?;
    }
    enforce_bound(output)
}

/// Render a minimal XML view with escaped authorized text and citation IDs.
///
/// # Errors
///
/// Returns [`ContextRenderError`] when the bundle violates the bounded
/// rendering contract.
pub fn render_xml(bundle: &ContextBundle) -> Result<String, ContextRenderError> {
    let mut output = format!(
        "<context plan=\"{}\" as_of=\"{}\" receipt=\"{}\">",
        escape(&bundle.plan_digest),
        escape(&bundle.as_of.to_rfc3339()),
        escape(&bundle.receipt_digest)
    );
    for memory in &bundle.memories {
        write!(
            &mut output,
            "<memory id=\"{}\">{}</memory>",
            escape(memory.id.as_str()),
            escape(&memory.content.text)
        )
        .map_err(|_| ContextRenderError::Formatting)?;
    }
    for redacted in &bundle.redacted {
        write!(
            &mut output,
            "<memory id=\"{}\" redacted=\"true\"/>",
            escape(redacted.id.as_str())
        )
        .map_err(|_| ContextRenderError::Formatting)?;
    }
    output.push_str("</context>");
    enforce_bound(output)
}

fn enforce_bound(output: String) -> Result<String, ContextRenderError> {
    if output.len() > MAX_CONTEXT_RENDER_BYTES {
        Err(ContextRenderError::OutputTooLarge)
    } else {
        Ok(output)
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
