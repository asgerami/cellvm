//! P1 Verify: establish invariants for P2.

pub mod frame;
pub mod typeflow;

use crate::error::Result;
use crate::module::Module;

pub fn verify_module(m: &Module) -> Result<()> {
    let f = m.entry().ok_or(crate::error::Error::VerifyFailed("no entry"))?;
    frame::FrameVerifier::new(f).verify()?;
    typeflow::TypeFlow::verify_function(f)?;
    Ok(())
}
