//! Integer interval lattice for abstract interpretation helpers.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    pub lo: i64,
    pub hi: i64,
}

impl Interval {
    pub fn point(x: i64) -> Self {
        Self { lo: x, hi: x }
    }

    pub fn top() -> Self {
        Self { lo: i64::MIN / 4, hi: i64::MAX / 4 }
    }

    pub fn contains(self, x: i64) -> bool {
        self.lo <= x && x <= self.hi
    }

    pub fn join(self, other: Self) -> Self {
        Self { lo: self.lo.min(other.lo), hi: self.hi.max(other.hi) }
    }

    pub fn meet(self, other: Self) -> Option<Self> {
        let lo = self.lo.max(other.lo);
        let hi = self.hi.min(other.hi);
        if lo <= hi {
            Some(Self { lo, hi })
        } else {
            None
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            lo: self.lo.saturating_add(other.lo),
            hi: self.hi.saturating_add(other.hi),
        }
    }

    pub fn widen(self, other: Self) -> Self {
        Self {
            lo: if other.lo < self.lo { i64::MIN / 4 } else { self.lo },
            hi: if other.hi > self.hi { i64::MAX / 4 } else { self.hi },
        }
    }
}

pub fn from_push8(v: i8) -> Interval {
    Interval::point(v as i64)
}
