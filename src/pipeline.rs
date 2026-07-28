//! Load → Verify → Link/Intern → Execute pipeline.

use crate::deser;
use crate::error::Result;
use crate::exec;
use crate::intern;
use crate::loader;
use crate::module::Module;
use crate::passes;
use crate::session::Session;
use crate::validate_extra;
use crate::verifier;

#[derive(Clone, Copy, Debug, Default)]
pub struct LoadRunOpts {
    pub run_exec: bool,
}

pub fn load_run(input: &[u8], opts: LoadRunOpts) -> Result<()> {
    let module = loader::load_module(input)?;
    run_module(&module, opts)
}

pub fn run_module(module: &Module, opts: LoadRunOpts) -> Result<()> {
    let mut session = Session::default();
    session.note_load();
    verifier::verify_module(module)?;
    validate_extra::validate_module_extra(module)?;
    passes::run_default(&mut session, module);
    intern::link_module_unit(module)?;
    if opts.run_exec {
        session.note_exec();
        exec::run_module_safe(module)?;
    }
    let _ = session.summary();
    Ok(())
}

pub fn load_verify_only(input: &[u8]) -> Result<Module> {
    let m = loader::load_module(input)?;
    verifier::verify_module(&m)?;
    validate_extra::validate_module_extra(&m)?;
    Ok(m)
}

pub fn deser_teardown(blob: &[u8]) -> Result<()> {
    deser::deserialize_and_drop(blob)
}
