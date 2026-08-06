//! Marciana-owned cognition composition primitives.

mod binding_error;
mod engine_binding;
mod memory_error;

pub use binding_error::CognitionBindingError;
pub use engine_binding::CognitionEngineBinding;
pub use memory_error::CognitionMemoryError;

#[cfg(test)]
mod binding_error_tests;
#[cfg(test)]
mod engine_binding_tests;
#[cfg(test)]
mod memory_error_tests;
