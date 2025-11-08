//! Compatibility layer for aurora_effects integration

use crate::diagnostic::DiagnosticCollector;
use std::sync::Arc;

/// Wrapper to implement aurora_effects::DiagnosticCollector
pub struct EffectsDiagnosticAdapter {
    inner: Arc<DiagnosticCollector>,
}

impl EffectsDiagnosticAdapter {
    pub fn new(collector: Arc<DiagnosticCollector>) -> Self {
        Self { inner: collector }
    }
}

impl aurora_effects::DiagnosticCollector for EffectsDiagnosticAdapter {
    fn report_advisory(&self, message: String, _location: usize, _severity: u8) {
        // For now, just print to stderr (the effects checker is fully tested internally)
        // TODO: Properly integrate with diagnostic collector using interior mutability
        eprintln!("[ADVISORY] {}", message);
    }
}
