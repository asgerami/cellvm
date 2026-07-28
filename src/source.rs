//! Optional source-map side table (assembler → bytecode PC).

use std::collections::BTreeMap;

use crate::span::Span;

#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    pub file: String,
    pub pc_to_line: BTreeMap<u32, u32>,
    pub regions: Vec<(Span, u32)>,
}

impl SourceMap {
    pub fn new(file: impl Into<String>) -> Self {
        Self { file: file.into(), ..Default::default() }
    }

    pub fn define(&mut self, pc: usize, line: u32) {
        self.pc_to_line.insert(pc as u32, line);
    }

    pub fn region(&mut self, start: usize, end: usize, line: u32) {
        self.regions.push((Span::new(start, end), line));
    }

    pub fn line_of(&self, pc: usize) -> Option<u32> {
        if let Some(l) = self.pc_to_line.get(&(pc as u32)) {
            return Some(*l);
        }
        self.regions
            .iter()
            .find(|(s, _)| s.contains(pc))
            .map(|(_, l)| *l)
    }

    pub fn render_loc(&self, pc: usize) -> String {
        match self.line_of(pc) {
            Some(l) => format!("{}:{}", self.file, l),
            None => format!("{}:?", self.file),
        }
    }
}
