//! Extra static analyses used by tooling and optional optimize pipelines.

pub mod folding;
pub mod sinking;
pub mod hoisting;
pub mod strength;
pub mod cse;
pub mod dce_extra;
pub mod inline_hint;
pub mod specialize;
pub mod escape;
pub mod alias;
pub mod bounds;
pub mod nullness;
pub mod parity;
pub mod sign;
pub mod range;
pub mod taint;
pub mod effects;
pub mod purity;
pub mod layout;
pub mod schedule;
