//! Source/bytecode spans for diagnostics.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start: start as u32, end: end as u32 }
    }

    pub fn len(self) -> u32 { self.end.saturating_sub(self.start) }

    pub fn contains(self, pc: usize) -> bool {
        (pc as u32) >= self.start && (pc as u32) < self.end
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn overlap(self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpanMap {
    entries: Vec<(Span, &'static str)>,
}

impl SpanMap {
    pub fn push(&mut self, span: Span, label: &'static str) {
        self.entries.push((span, label));
    }

    pub fn label_at(&self, pc: usize) -> Option<&'static str> {
        self.entries.iter().find(|(s, _)| s.contains(pc)).map(|(_, l)| *l)
    }

    pub fn covering(&self, pc: usize) -> Vec<&'static str> {
        self.entries.iter().filter(|(s, _)| s.contains(pc)).map(|(_, l)| *l).collect()
    }
}
