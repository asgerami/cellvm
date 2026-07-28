//! Structured diagnostic reports for verify/load failures.

use crate::error::Error;
use crate::span::Span;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn from_error(err: &Error) -> Self {
        let (code, span) = match err {
            Error::UnknownOpcode { pc, .. } => ("E_OPCODE", Some(Span::new(*pc, *pc + 1))),
            Error::TruncatedInstr { pc } => ("E_TRUNC", Some(Span::new(*pc, *pc + 1))),
            Error::LocalOutOfRange { pc, .. } => ("E_LOCAL", Some(Span::new(*pc, *pc + 3))),
            Error::EffectiveSlotOutOfRange { pc, .. } => ("E_SLOT", Some(Span::new(*pc, *pc + 3))),
            Error::WindowUnderflow { pc } => ("E_WINDOW", Some(Span::new(*pc, *pc + 2))),
            Error::UntypedFieldAccess { .. } => ("E_TYPE", None),
            Error::OffsetTruncation { .. } => ("E_JMP", None),
            Error::TargetOutOfRange { .. } | Error::TargetNotBoundary { .. } => ("E_TARGET", None),
            Error::DeserFailed(_) => ("E_DESER", None),
            _ => ("E_GENERIC", None),
        };
        Self {
            code,
            message: format!("{err}"),
            span,
            notes: Vec::new(),
        }
    }

    pub fn note(mut self, n: impl Into<String>) -> Self {
        self.notes.push(n.into());
        self
    }

    pub fn render(&self) -> String {
        let mut s = format!("[{}] {}", self.code, self.message);
        if let Some(span) = self.span {
            s.push_str(&format!(" @{}..{}", span.start, span.end));
        }
        for n in &self.notes {
            s.push_str(&format!("\n  note: {n}"));
        }
        s
    }
}

pub fn render_many(diags: &[Diagnostic]) -> String {
    diags.iter().map(|d| d.render()).collect::<Vec<_>>().join("\n")
}
