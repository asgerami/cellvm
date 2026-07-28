//! Conservative mark helpers over the value heap registry.

use crate::heap::Registry;
use crate::value::Obj;

#[derive(Clone, Debug, Default)]
pub struct GcStats {
    pub marked: usize,
    pub swept: usize,
    pub bytes_reclaimed: usize,
}

pub fn mark_from_roots(reg: &Registry, roots: &[usize]) -> Vec<bool> {
    let n = reg.len();
    let mut marked = vec![false; n];
    for &i in roots {
        if i < n {
            marked[i] = true;
        }
    }
    // Objs currently have no child edges in the compact representation;
    // treat registered roots as the live set.
    for i in 0..n {
        if reg.get(i).is_some() && roots.contains(&i) {
            marked[i] = true;
        }
    }
    marked
}

pub fn count_live(marked: &[bool]) -> usize {
    marked.iter().filter(|m| **m).count()
}

pub fn estimate_bytes(obj: &Obj) -> usize {
    std::mem::size_of_val(obj)
}

pub fn summarize_heap(reg: &Registry) -> GcStats {
    let roots: Vec<usize> = (0..reg.len()).filter(|&i| reg.get(i).is_some()).collect();
    let marked = mark_from_roots(reg, &roots);
    GcStats {
        marked: count_live(&marked),
        swept: marked.iter().filter(|m| !**m).count(),
        bytes_reclaimed: 0,
    }
}

pub fn nursery_threshold(live: usize) -> usize {
    (live * 2).max(64).min(4096)
}
