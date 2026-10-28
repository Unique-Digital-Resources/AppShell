//! Sizing preferences: logical size state, independent of pixels.
//!
//! Row and column children use relative sizing weights (PRD §9.2 baseline).
//! Weights express the intended distribution of available space among
//! siblings; the frontend derives the actual geometry from the logical
//! state and the available space. The environment never requires a
//! browser measurement to validate a structural operation (PRD §9.1).
//!
//! ## The stored-weight representation decision (PRD §9.2, Phase 6)
//!
//! Weights are stored **verbatim and never renormalized**. The only
//! representation invariants are the construction-guaranteed ones: one
//! finite, strictly positive weight per child. There is deliberately no
//! sum ≈ 1 constraint:
//!
//! - Weights are *relative* shares — the rendering proportion of a child
//!   is the derived read `weightᵢ / Σweightⱼ`, computed by the frontend;
//!   it is never stored. A stored set sums to whatever the operations
//!   left it at (`uniform(n)` sums to `n`; an inserted child copying the
//!   target's weight grows the sum — both equally valid relative sets).
//! - The transfer rules fixed in Phases 4–5 would be invalidated by
//!   write-time rescaling: removal keeps survivors' shares verbatim
//!   (`prune_weight` never rescales) and collapse keeps the parent's
//!   slot weight, so stored values stay stable under the structure
//!   changes they already survived.
//! - Rescaling on every write would churn every untouched weight and
//!   make undo/redo restore comparisons drift against the stored text;
//!   a resize instead rewrites exactly the two regions it touches (see [`ResizeSiblings`]).
//!
//! Initialized, updated, normalized, transferred (PRD §9.2):
//!
//! | Stage | Policy |
//! |---|---|
//! | initialized | fresh containers start uniform; a child inserted beside a target copies the target's weight (insertion flows, Phase 4) |
//! | updated | only by [`ResizeSiblings`] — the adjacent pair is rewritten, every other child's weight verbatim |
//! | normalized | never at rest; the normalized proportion is a derived read, not stored state |
//! | transferred | removal deletes the departed child's weight (survivors verbatim, Phase 5); collapse leaves the slot's weight with the surviving child; collapse/restore toggles change no weighting at all (PRD §9.7) |
//!
//! Structural transfer baselines are documented in `normalization.rs`
//! (the removal/collapse policy) and the operation modules. Minimum size
//! bounds are translated by the frontend into per-operation constraints
//! (PRD §9.3); they are not stored model state. Floating-dock placement
//! and sizing preferences are the §9.6 floating contract (Phase 8,
//! [`floating`](super::floating)) — one persistent logical rect per
//! entry, stored verbatim, rewritten only by `SetFloatingRect` — and
//! shared-junction resizing is deferred by the PRD (§9.8).
//!
//! ## Embedded-region sizing (PRD §9.5, Phase 7)
//!
//! Each **occupied** edge region of a host carries one persistent
//! sizing weight, stored verbatim under this module's representation
//! decision (finite, strictly positive; never renormalized — the
//! proportion is the derived read `weightᵢ / Σweightⱼ` of its stage).
//! The weight attaches to the **slot** (parent-side, like
//! `Split { children, sizes }`), mirroring the split transfer table:
//!
//! | Stage | Policy |
//! |---|---|
//! | initialized | an embed into a fresh region starts at the embedding weight `1.0` (uniform with the implicit core) |
//! | updated | only by `ResizeRegion` — exactly the named region's weight is rewritten; the host's other placements and stored weights stay verbatim |
//! | transferred | slot inheritance keeps the weight: a drop beside a region occupant wraps the region into a split that inherits occupant *and* weight; a collapsing split's survivor inherits the region slot and weight |
//! | cleared | the weight lifecycle follows occupancy — clearing the region resets the slot to the embedding baseline; weights live with the region lifecycle |
//!
//! The **allocation definition** (how occupied regions and the host's
//! content divide the host — PRD §9.5's top/bottom ↔ left/right
//! interaction) is a derived two-stage read, documented in
//! [`embed_host`](super::embed_host): the column stage weights
//! top/core(1)/bottom; the row
//! stage inside the core band weights left/content(1)/right. Resizing
//! is therefore a boundary between a region and the core — one region's
//! weight per request — and the core holds no stored weight in v1 (it
//! is the implicit complement; a content-side minimum is a MAXimum on
//! the region, part of the §9.3 MAY that v1 one-sided bounds omit).
//!
//! Region resizing shares the `ResizeSiblings` gesture and constraint
//! contracts (absolute drift-free requests; operation-carried minimums;
//! rejection, never clamping — [`RegionBoundViolation`](super::error::DockingError::RegionBoundViolation)).
//!
//! [`ResizeSiblings`]:
//!     super::operations::DockingOperation::ResizeSiblings

use std::fmt;

/// Relative sizes for the children of a split container.
///
/// Invariants guaranteed by construction:
/// - one weight per child (matching the container's child count),
/// - every weight is finite and strictly positive.
///
/// Stored weights are **kept verbatim** — never renormalized to a
/// target sum (the representation decision, see the module docs). The
/// rendering proportion of a child is the derived read
/// `weightᵢ / Σweightⱼ`; the sum itself is not an invariant.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SplitSizes(Vec<f64>);

impl SplitSizes {
    /// A uniform distribution over `count` children.
    ///
    /// Returns `None` when `count == 0`; an empty split container is
    /// not a valid steady state (see normalization policy, PRD §8.12).
    pub fn uniform(count: usize) -> Option<Self> {
        if count == 0 {
            return None;
        }
        Some(Self(vec![1.0; count]))
    }

    /// A distribution from explicit weights.
    ///
    /// Returns `None` unless every weight is finite and strictly positive.
    /// The vector is stored as given; empty input yields an empty set
    /// (structurally valid only for re-combination inside operations).
    pub fn from_weights(weights: Vec<f64>) -> Option<Self> {
        if weights.iter().any(|w| !w.is_finite() || *w <= 0.0) {
            return None;
        }
        Some(Self(weights))
    }

    /// Number of weights (must match the container's child count).
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` when there are no weights.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The weights in child order.
    pub fn weights(&self) -> &[f64] {
        &self.0
    }

    /// Whether the weight count matches `count` children.
    pub fn matches(&self, count: usize) -> bool {
        self.0.len() == count
    }

    /// Insert `weight` at `index` (operation-side helper for
    /// insert-beside: the new child receives the copied target weight,
    /// which the operations module guarantees to be finite and strictly
    /// positive, PRD §9.2).
    pub(crate) fn insert_weight(&mut self, index: usize, weight: f64) {
        self.0.insert(index, weight);
    }

    /// Test-only constructor that skips the finite / strictly-positive
    /// checks, letting validation fixtures build *invalid* sizing state
    /// the way an imported (restored) layout could carry it. Not part of
    /// the public API; real weights can always be built through
    /// [`from_weights`](Self::from_weights) / [`uniform`](Self::uniform).
    #[cfg(test)]
    pub(crate) fn from_weights_unchecked(weights: Vec<f64>) -> Self {
        Self(weights)
    }
}

impl fmt::Display for SplitSizes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, w) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{w:.3}")?;
        }
        write!(f, "]")
    }
}

impl IntoIterator for SplitSizes {
    type Item = f64;
    type IntoIter = std::vec::IntoIter<f64>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
