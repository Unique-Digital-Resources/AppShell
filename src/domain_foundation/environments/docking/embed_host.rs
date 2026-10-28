//! Embed-host state: occupied edge regions with their sizing
//! preferences, and the floating-dock collection.
//!
//! An embed host retains its own work area (its dock's [`ContentRef`])
//! while providing one occupied dock subtree per edge region and any
//! number of floating entries (PRD §7.2, §8.8). Region collections are
//! controlled by the [`DockLayout`](super::model::DockLayout); external
//! code reads them through the accessors below.
//!
//! ## Region sizing (PRD §9.5)
//!
//! Each occupied edge region carries one **persistent sizing weight**,
//! stored verbatim under the same representation decision as split
//! weights (PRD §9.2 — see [`sizing`](super::sizing)): finite and
//! strictly positive, never renormalized; the proportion is a derived
//! frontend read. The weight attaches to the **slot**, not the occupant
//! — the same parent-side representation as `Split { children, sizes }`
//! — so it is arrangement state, not dock state: it is established when
//! the region is embedded into (baseline `1.0`, the uniform-with-core
//! default), inherited unchanged by slot rewrites (a drop beside a
//! region occupant wraps the region into a split that keeps the slot's
//! weight), rewritten only by `ResizeRegion`, and reset to the baseline
//! when the region is cleared (weights live with the region lifecycle).
//!
//! The allocation definition (how much space each occupied region and
//! the host's content get — PRD §9.5) is a derived two-stage read:
//!
//! 1. **Column stage** — the participants are the weights of the
//!    occupied `top`/`bottom` regions plus the implicit core weight
//!    `1.0`: `top` takes `w_top / (w_top + 1 + w_bottom)` of the host's
//!    height, `bottom` its proportional share, the core band the
//!    remainder. Top and bottom therefore interact only through the
//!    shared core — resizing one never rewrites the other's stored
//!    weight.
//! 2. **Row stage** — inside the core band, the occupied
//!    `left`/`right` weights plus the implicit weight `1.0` of the
//!    content cell divide the band's width the same way.
//!
//! Adjusting the logical boundary between a region and the host's
//! content is a one-region rewrite: the frontend translates the
//! boundary gesture into the region's next **absolute** weight (the
//! same drift-free gesture contract as [`ResizeSiblings`](super::operations::DockingOperation::ResizeSiblings)).
//! The core itself holds no stored weight in v1 — it is the implicit
//! complement, so a core-side minimum expresses as the region's carried
//! policy, consistent with the one-sided bounds of PRD §9.3 (v1
//! minimums only; maxima MAY).
//!
//! Floating entries carry their position/size state as the **§9.6
//! floating contract** (Phase 8): one persistent logical rect per
//! entry (fractions of the host's body), defined in
//! [`floating`](super::floating) alongside the entry type.

use super::dock::DockId;
use super::slot::EdgeRegion;

/// The embedding baseline: the weight a freshly occupied region starts
/// at (uniform with the implicit core weight `1.0`, PRD §9.5).
pub(crate) const REGION_WEIGHT_BASELINE: f64 = 1.0;

/// The four occupied edge-region placements of an embed host, each with
/// its persistent sizing weight (PRD §8.8, §9.5).
///
/// Each region holds at most one dock subtree (PRD §8.8 initial-version
/// baseline). `None` means the region is empty. A region's weight is
/// meaningful exactly while the region is occupied (`weight_on`); the
/// stored field resets to the embedding baseline when the occupant is
/// detached, so an empty region contributes the default the next embed
/// starts from.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))] // absent weight fields restore at the baseline
#[derive(Debug, Clone, PartialEq)]
pub struct HostRegions {
    pub(crate) top: Option<DockId>,
    pub(crate) bottom: Option<DockId>,
    pub(crate) left: Option<DockId>,
    pub(crate) right: Option<DockId>,
    /// Persistent sizing weights per edge (PRD §9.5): meaningful iff
    /// the edge is occupied; the stored value of an empty region is the
    /// inert embedding baseline. Managed only through [`assign` /
    /// `set_weight`](HostRegions::assign) (operation territory).
    top_weight: f64,
    bottom_weight: f64,
    left_weight: f64,
    right_weight: f64,
}

impl Default for HostRegions {
    fn default() -> Self {
        Self {
            top: None,
            bottom: None,
            left: None,
            right: None,
            top_weight: REGION_WEIGHT_BASELINE,
            bottom_weight: REGION_WEIGHT_BASELINE,
            left_weight: REGION_WEIGHT_BASELINE,
            right_weight: REGION_WEIGHT_BASELINE,
        }
    }
}

impl HostRegions {
    /// An empty region set (every region unoccupied at the baseline
    /// weight).
    pub fn new() -> Self {
        Self::default()
    }

    /// The dock occupying `edge`, if any.
    pub fn on(&self, edge: EdgeRegion) -> Option<DockId> {
        match edge {
            EdgeRegion::Top => self.top,
            EdgeRegion::Bottom => self.bottom,
            EdgeRegion::Left => self.left,
            EdgeRegion::Right => self.right,
        }
    }

    /// The sizing weight of `edge` while it is occupied (PRD §9.5);
    /// `None` for an empty region — the stored baseline of an empty
    /// region is not a preference (weights live with the region
    /// lifecycle).
    pub fn weight_on(&self, edge: EdgeRegion) -> Option<f64> {
        match edge {
            EdgeRegion::Top => self.top.is_some().then_some(self.top_weight),
            EdgeRegion::Bottom => self.bottom.is_some().then_some(self.bottom_weight),
            EdgeRegion::Left => self.left.is_some().then_some(self.left_weight),
            EdgeRegion::Right => self.right.is_some().then_some(self.right_weight),
        }
    }

    /// The weight the edge's slot currently holds — the occupant's
    /// weight for an occupied edge, the embedding baseline for an empty
    /// one (what a slot-inheriting rewrite carries forward and what the
    /// next embed starts from). `pub(crate)`: operation-side read.
    pub(crate) fn slot_weight(&self, edge: EdgeRegion) -> f64 {
        self.weight_on(edge).unwrap_or(REGION_WEIGHT_BASELINE)
    }

    /// Assign `dock` (and its sizing `weight`) to `edge`, replacing any
    /// previous occupant. Clearing resets the slot's weight to the
    /// embedding baseline (weights live with the region lifecycle).
    ///
    /// `pub(crate)`: occupancy decisions are made by docking operations,
    /// which implement the explicit replace/exchange policy (PRD §8.8);
    /// ad-hoc mutation from outside the environment is not allowed.
    pub(crate) fn assign(&mut self, edge: EdgeRegion, dock: Option<DockId>, weight: f64) {
        match edge {
            EdgeRegion::Top => {
                self.top_weight = weight;
                self.top = dock;
            }
            EdgeRegion::Bottom => {
                self.bottom_weight = weight;
                self.bottom = dock;
            }
            EdgeRegion::Left => {
                self.left_weight = weight;
                self.left = dock;
            }
            EdgeRegion::Right => {
                self.right_weight = weight;
                self.right = dock;
            }
        }
        if dock.is_none() {
            // The cleared slot resets to the embed baseline (the weight
            // lifecycle keeps empty regions at the default the next
            // embed starts from).
            self.assign_weight(edge, REGION_WEIGHT_BASELINE);
        }
    }

    /// Rewrite an occupied region's sizing weight in place (the
    /// `ResizeRegion` write — the occupant is untouched, the region's
    /// other placements keep their stored weights verbatim, PRD §9.5).
    pub(crate) fn set_weight(&mut self, edge: EdgeRegion, weight: f64) {
        match edge {
            EdgeRegion::Top => self.top_weight = weight,
            EdgeRegion::Bottom => self.bottom_weight = weight,
            EdgeRegion::Left => self.left_weight = weight,
            EdgeRegion::Right => self.right_weight = weight,
        }
    }

    /// The weight-only write under `assign`'s clearing branch.
    fn assign_weight(&mut self, edge: EdgeRegion, weight: f64) {
        match edge {
            EdgeRegion::Top => self.top_weight = weight,
            EdgeRegion::Bottom => self.bottom_weight = weight,
            EdgeRegion::Left => self.left_weight = weight,
            EdgeRegion::Right => self.right_weight = weight,
        }
    }

    /// A placement slot is occupied by the given dock somewhere on the
    /// host — returned edge identifies which region, if any.
    pub fn owner_of(&self, dock: DockId) -> Option<EdgeRegion> {
        [
            (EdgeRegion::Top, self.top),
            (EdgeRegion::Bottom, self.bottom),
            (EdgeRegion::Left, self.left),
            (EdgeRegion::Right, self.right),
        ]
        .into_iter()
        .find(|(_, occupant)| *occupant == Some(dock))
        .map(|(edge, _)| edge)
    }

    /// The occupied edges, in fixed scan order: top, bottom, left, right.
    pub fn occupied(&self) -> Vec<EdgeRegion> {
        [
            EdgeRegion::Top,
            EdgeRegion::Bottom,
            EdgeRegion::Left,
            EdgeRegion::Right,
        ]
        .into_iter()
        .filter(|&edge| self.on(edge).is_some())
        .collect()
    }

    /// All docks occupying regions, in fixed scan order
    /// (top, bottom, left, right) — duplicates reported once per region.
    pub fn occupied_docks(&self) -> Vec<DockId> {
        vec![self.top, self.bottom, self.left, self.right]
            .into_iter()
            .flatten()
            .collect()
    }

    /// `true` when no region is occupied.
    pub fn is_empty(&self) -> bool {
        self.top.is_none() && self.bottom.is_none() && self.left.is_none() && self.right.is_none()
    }
}
