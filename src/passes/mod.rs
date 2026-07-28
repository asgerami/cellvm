//! Analysis pass driver.

pub mod coverage;
pub mod metrics;
pub mod normalize;

use crate::module::Module;
use crate::session::Session;

pub fn run_default(session: &mut Session, module: &Module) {
    session.note_verify();
    session.record_module_stats(module);
    let _ = metrics::module_metric_vector(module);
    let _ = coverage::opcode_coverage(module);
    let _ = normalize::normalized_hash(module);
}
