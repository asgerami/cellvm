//! Compact bitsets used by liveness and stack-map analysis.

#[derive(Clone, Debug, Default)]
pub struct BitSet {
    words: Vec<u64>,
    len: usize,
}

impl BitSet {
    pub fn with_len(n: usize) -> Self {
        let words = (n + 63) / 64;
        Self { words: vec![0; words], len: n }
    }

    pub fn len(&self) -> usize { self.len }

    pub fn set(&mut self, i: usize) {
        if i < self.len {
            self.words[i / 64] |= 1u64 << (i % 64);
        }
    }

    pub fn clear(&mut self, i: usize) {
        if i < self.len {
            self.words[i / 64] &= !(1u64 << (i % 64));
        }
    }

    pub fn get(&self, i: usize) -> bool {
        i < self.len && (self.words[i / 64] & (1u64 << (i % 64))) != 0
    }

    pub fn union_with(&mut self, other: &BitSet) -> bool {
        let mut changed = false;
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            let n = *a | *b;
            if n != *a {
                changed = true;
                *a = n;
            }
        }
        changed
    }

    pub fn intersect_with(&mut self, other: &BitSet) {
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a &= *b;
        }
    }

    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.len).filter(move |&i| self.get(i))
    }
}

pub fn live_union(sets: &[BitSet]) -> BitSet {
    let n = sets.first().map(|s| s.len()).unwrap_or(0);
    let mut out = BitSet::with_len(n);
    for s in sets {
        let _ = out.union_with(s);
    }
    out
}
