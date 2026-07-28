//! Constant folding / evaluation utilities over decoded instruction streams.

use crate::decode::{self, Decoded};
use crate::error::{Error, Result};
use crate::module::Function;

#[derive(Clone, Debug, Default)]
pub struct EvalFrame {
    pub stack: Vec<i64>,
    pub locals: Vec<i64>,
}

impl EvalFrame {
    pub fn new(nlocals: usize) -> Self {
        Self {
            stack: Vec::new(),
            locals: vec![0; nlocals],
        }
    }

    pub fn push(&mut self, v: i64) {
        self.stack.push(v);
    }

    pub fn pop(&mut self) -> Result<i64> {
        self.stack.pop().ok_or(Error::VerifyFailed("stack underflow"))
    }

    pub fn peek(&self) -> Option<i64> {
        self.stack.last().copied()
    }
}

pub fn eval_straight(f: &Function, max_steps: usize) -> Result<EvalFrame> {
    let mut frame = EvalFrame::new(f.max_locals as usize);
    let mut pc = 0usize;
    let mut steps = 0usize;
    while pc < f.code.len() && steps < max_steps {
        let (d, len) = decode::decode_at(&f.code, pc)?;
        match d {
            Decoded::Nop => {}
            Decoded::Push8(v) => frame.push(v as i64),
            Decoded::LoadK(i) => {
                let v = *f.consts.get(i as usize).ok_or(Error::VerifyFailed("const"))?;
                frame.push(v);
            }
            Decoded::LoadLocal(i) => {
                let v = *frame.locals.get(i as usize).unwrap_or(&0);
                frame.push(v);
            }
            Decoded::StoreLocal(i) => {
                let v = frame.pop()?;
                if let Some(slot) = frame.locals.get_mut(i as usize) {
                    *slot = v;
                }
            }
            Decoded::Window(_) => {}
            Decoded::Ret | Decoded::Throw => break,
            Decoded::Sjmp(off) => {
                let next = (pc as i64 + len as i64 + off as i64) as usize;
                pc = next;
                steps += 1;
                continue;
            }
            Decoded::StrCmp => {
                let _ = frame.pop()?;
                frame.push(0);
            }
            Decoded::NewArr(_) => frame.push(0xA1),
            Decoded::GetField(_) => {
                let _ = frame.pop()?;
                frame.push(0);
            }
            Decoded::Call(_) | Decoded::Tail(_) => {
                // interpreter stub: leave stack alone
            }
            Decoded::Intern(_) => frame.push(0),
            Decoded::Close(_) | Decoded::OpenUp(_) | Decoded::GetUp(_) | Decoded::SetUp(_) => {}
            Decoded::Unknown(_) => return Err(Error::VerifyFailed("unknown op")),
        }
        pc += len;
        steps += 1;
    }
    Ok(frame)
}

pub fn stack_height_trace(f: &Function) -> Result<Vec<(usize, i32)>> {
    let mut depth = 0i32;
    let mut out = Vec::new();
    let mut pc = 0usize;
    while pc < f.code.len() {
        let (d, len) = decode::decode_at(&f.code, pc)?;
        out.push((pc, depth));
        depth += decode::stack_delta_of(&d) as i32;
        if depth < 0 {
            return Err(Error::VerifyFailed("neg depth"));
        }
        pc += len;
    }
    Ok(out)
}

pub fn max_stack_height(f: &Function) -> Result<i32> {
    Ok(stack_height_trace(f)?
        .into_iter()
        .map(|(_, d)| d)
        .max()
        .unwrap_or(0))
}

pub fn count_returns(f: &Function) -> Result<usize> {
    let decoded = decode::decode_all(&f.code)?;
    Ok(decoded
        .into_iter()
        .filter(|(_, d)| matches!(d, Decoded::Ret | Decoded::Throw | Decoded::Tail(_)))
        .count())
}

pub fn has_loops(f: &Function) -> Result<bool> {
    let decoded = decode::decode_all(&f.code)?;
    for (pc, d) in decoded {
        if let Decoded::Sjmp(off) = d {
            if (pc as i64 + 2 + off as i64) < pc as i64 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub fn summarize_function(f: &Function) -> Result<String> {
    Ok(format!(
        "{} code={} max_stack={} returns={} loops={}",
        f.name,
        f.code.len(),
        max_stack_height(f)?,
        count_returns(f)?,
        has_loops(f)?,
    ))
}
