//! Docking-specific errors.
//!
//! The environment owns a single error type, in the style of the other
//! Domain Foundation environments. Variants are added as behaviour is
//! implemented (structural validation and layout-invariant errors in
//! Phase 3, further operation errors in Phase 4); the style —
//! serde-gated, `Display`, `std::error::Error` — is fixed here.

use super::dock::DockId;
use super::slot::EdgeRegion;
use std::fmt;

/// Errors produced by the docking environment.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockingError {
    /// No dock exists in the layout with the given id.
    DockNotFound { id: DockId },
    /// A created placement references a dock that does not exist
    /// in the layout.
    UnknownReference { id: DockId },
    /// The same dock appears more than once in one parent's children,
    /// members, regions, or floating collection.
    DuplicateMembership { id: DockId },
    /// A dock that already occupies a placement was used as if it were
    /// unplaced (PRD DM-07: every dock has exactly one placement).
    AlreadyPlaced { id: DockId },
    /// A structural group was created with fewer children/members than
    /// its minimum: splits need ≥ 2 children, tab groups need ≥ 1 member
    /// (creation-time precondition; normalization policy, PRD §8.12).
    InsufficientChildren { minimum: usize, found: usize },
    /// The size distribution's weight count does not match the
    /// container's child count (PRD §9.2).
    SizeCountMismatch { sizes: usize, children: usize },
    /// The active-tab index is not a valid member position
    /// (PRD §7.2, active-tab validity).
    ActiveIndexOutOfRange { index: usize, member_count: usize },
    /// The layout root is already set; clear it before designating a new
    /// root. Phase 2 keeps root changes explicit instead of implicitly
    /// re-rooting.
    RootAlreadySet { current: DockId },

    // --- Layout invariants (structural validation, Phase 3) ---
    /// A dock exists in the layout but occupies no placement: it is
    /// neither the root nor referenced by any container collection
    /// (PRD DM-07: every dock has exactly one placement). Valid only as
    /// a transient mid-operation state; committed and imported layouts
    /// must be orphan-free.
    OrphanDock { id: DockId },
    /// A dock occupies more than one placement: two or more container
    /// slots reference it (possibly the same dock listed twice in one
    /// collection, or separate containers both claiming it). The
    /// placement scan owns exactly one slot per dock in a valid layout
    /// (PRD DM-07).
    DuplicatePlacement { id: DockId },
    /// The designated root is also placed inside a container. The root
    /// is the top-level dock and must have no owner: its only correct
    /// placement is the [`Root`](super::slot::DockSlot::Root) slot.
    InvalidRoot { id: DockId },
    /// The layout contains docks but designates no root. Only the
    /// explicit empty layout (no docks, no root) or a transient
    /// mid-operation state may be rootless (PRD DM-06, §7.4).
    MissingRoot { docks: usize },
    /// A structural reference cycle exists among docks. Cycles are
    /// detectable through any placement category: split children, tab
    /// members, host edge regions, and floating collections all count
    /// as containment (PRD §10.4; naming follows the
    /// `LayersError::CycleDetected` precedent).
    CycleDetected { id: DockId },
    /// A split container's stored sizing state is invalid: the weight at
    /// `index` is not finite or not strictly positive (PRD §9.2). This is
    /// reachable only for imported (restored) state — constructor-gated
    /// sizes cannot hold such weights.
    InvalidSizeState { split: DockId, index: usize },

    // --- Operations (Phase 4) ---
    /// A dock's category does not qualify it for the requested operation:
    /// tab operations accept panels only (PRD §8.7, v1 membership rule).
    IneligibleDock { id: DockId },
    /// The requested directional insertion has no defined structure at
    /// the target's placement: dropping beside a dock in a floating
    /// collection has no row/column meaning in v1 (PRD §8.6 baseline).
    NotSplittable { id: DockId },
    /// The given id names a dock that is not a tab group (a tab-group
    /// argument for reorder or active-selection, PRD §8.7).
    NotATabGroup { id: DockId },
    /// A tab member index is outside the group's member list (a
    /// reorder argument; the active-selection rule keeps
    /// [`ActiveIndexOutOfRange`](DockingError::ActiveIndexOutOfRange)).
    TabIndexOutOfRange { index: usize, member_count: usize },

    // --- Sizing operations (Phase 6) ---
    /// The given id names a dock that is not a split container (a
    /// resize argument; the split-side sibling of
    /// [`NotATabGroup`](DockingError::NotATabGroup)).
    NotASplit { id: DockId },
    /// A resize's pair-start child index has no adjacent sibling: the
    /// adjacent pair [`first`, `first` + 1] would run past the
    /// container's child list (a resize argument).
    SplitChildIndexOutOfRange { index: usize, child_count: usize },
    /// A resize request would assign a region a weight outside its
    /// logical bounds: the operation's carried minimum (PRD §9.3) or
    /// the sizing invariant itself (finite, strictly positive — PRD
    /// §9.2). The environment's constraint policy is **rejection** — a
    /// conflicting request is refused, never silently clamped or
    /// resolved into invalid sizing state. `index` names the region of
    /// the pair that is checked first (ascending child index) and
    /// violates its bound.
    SizeBoundViolation { split: DockId, index: usize },

    // --- Embedding and host lifecycle (Phase 7) ---
    /// The given id names a dock that is not an embed host (an embed or
    /// region-resize argument pointing at a split or tab group, which
    /// cannot host — panels auto-promote inside `Embed`, PRD §8.8–§8.9;
    /// the host-side sibling of
    /// [`NotATabGroup`](DockingError::NotATabGroup) /
    /// [`NotASplit`](DockingError::NotASplit)).
    NotAnEmbedHost { id: DockId },
    /// An embedding request targeted an edge region that is already
    /// occupied (PRD §8.8 baseline: the attempt is rejected — the
    /// existing occupant must never be silently discarded; a v1 embed
    /// defines no replacement/exchange policy).
    RegionOccupied { host: DockId, edge: EdgeRegion },
    /// A region-resize request named an edge region that is unoccupied
    /// (PRD §9.5: regions resize against their inner boundary — an
    /// empty region has no boundary and holds no sizing preference,
    /// weights live with the region lifecycle).
    RegionUnoccupied { host: DockId, edge: EdgeRegion },
    /// A region-resize request would assign the region a weight outside
    /// its logical bounds: the operation's carried minimum (PRD §9.3)
    /// or the sizing invariant itself (finite, strictly positive). Same
    /// rejection policy as [`SizeBoundViolation`](DockingError::SizeBoundViolation),
    /// naming the host and region instead of the split pair.
    RegionBoundViolation { host: DockId, edge: EdgeRegion },
    /// An embed host's stored region state is invalid: the weight of an
    /// occupied `edge` is not finite or not strictly positive (PRD
    /// §9.5; reachable only for imported/restored state — operation
    /// paths keep region weights valid).
    InvalidRegionState { host: DockId, edge: EdgeRegion },

    // --- Floating state (Phase 8) ---
    /// A floating-collection argument named an index outside the
    /// host's floating entries (a rect-set argument; the floating-side
    /// sibling of
    /// [`TabIndexOutOfRange`](DockingError::TabIndexOutOfRange) /
    /// [`SplitChildIndexOutOfRange`](DockingError::SplitChildIndexOutOfRange)).
    FloatingIndexOutOfRange { index: usize, entry_count: usize },
    /// A floating-rect request would place an entry outside its logical
    /// bounds: the operation's carried minimum (PRD §9.3), the rect
    /// invariant itself (finite, non-empty), or the host-bounds rule
    /// (`x + w ≤ 1`, `y + h ≤ 1` — PRD §9.6 SHOULD; endpoints
    /// inclusive). Same rejection policy as
    /// [`RegionBoundViolation`](DockingError::RegionBoundViolation) —
    /// never clamped, never invalid state stored.
    FloatingBoundViolation { host: DockId, index: usize },
    /// An embed host's stored floating state is invalid: the rect of
    /// the entry at `index` violates the rect invariant or the
    /// host-bounds rule (PRD §9.6; reachable only for
    /// imported/restored state — the `InvalidRegionState` mirror).
    InvalidFloatingState { host: DockId, index: usize },
}

impl fmt::Display for DockingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DockingError::DockNotFound { id } => write!(f, "no dock with id {id}"),
            DockingError::UnknownReference { id } => {
                write!(f, "dock reference {id} does not exist in the layout")
            }
            DockingError::DuplicateMembership { id } => {
                write!(f, "dock {id} appears more than once in one placement")
            }
            DockingError::AlreadyPlaced { id } => {
                write!(f, "dock {id} already occupies a placement")
            }
            DockingError::InsufficientChildren { minimum, found } => {
                write!(f, "structure expects at least {minimum} children, found {found}")
            }
            DockingError::SizeCountMismatch { sizes, children } => {
                write!(f, "size count {sizes} does not match child count {children}")
            }
            DockingError::ActiveIndexOutOfRange { index, member_count } => {
                write!(f, "active tab index {index} out of range (member count {member_count})")
            }
            DockingError::RootAlreadySet { current } => {
                write!(f, "root is already set to {current} (clear the root first)")
            }
            DockingError::OrphanDock { id } => {
                write!(f, "dock {id} has no placement (orphan)")
            }
            DockingError::DuplicatePlacement { id } => {
                write!(f, "dock {id} occupies more than one placement")
            }
            DockingError::InvalidRoot { id } => {
                write!(f, "dock {id} is the root but is also placed in a container")
            }
            DockingError::MissingRoot { docks } => {
                write!(f, "layout holds {docks} docks but designates no root")
            }
            DockingError::CycleDetected { id } => {
                write!(f, "structural cycle detected at {id}")
            }
            DockingError::InvalidSizeState { split, index } => {
                write!(
                    f,
                    "sizing weight {index} of {split} is invalid (must be finite and strictly positive)"
                )
            }
            DockingError::IneligibleDock { id } => {
                write!(f, "dock {id} does not qualify for this operation (v1 tab membership is panels only)")
            }
            DockingError::NotSplittable { id } => {
                write!(
                    f,
                    "dock {id} is in a floating placement: directional insertion is not defined for floating docks"
                )
            }
            DockingError::NotATabGroup { id } => {
                write!(f, "dock {id} is not a tab group")
            }
            DockingError::TabIndexOutOfRange { index, member_count } => {
                write!(f, "tab index {index} out of range (member count {member_count})")
            }
            DockingError::NotASplit { id } => {
                write!(f, "dock {id} is not a split container")
            }
            DockingError::SplitChildIndexOutOfRange { index, child_count } => {
                write!(f, "split child index {index} out of range (child count {child_count})")
            }
            DockingError::SizeBoundViolation { split, index } => {
                write!(
                    f,
                    "resize rejected: weight {index} of {split} would violate its logical size bounds"
                )
            }
            DockingError::NotAnEmbedHost { id } => {
                write!(f, "dock {id} is not an embed host")
            }
            DockingError::RegionOccupied { host, edge } => {
                write!(f, "{edge:?} region of {host} is already occupied (embedding never replaces an occupant)")
            }
            DockingError::RegionUnoccupied { host, edge } => {
                write!(f, "{edge:?} region of {host} is unoccupied and cannot be resized")
            }
            DockingError::RegionBoundViolation { host, edge } => {
                write!(
                    f,
                    "region resize rejected: the {edge:?} region of {host} would violate its logical size bounds"
                )
            }
            DockingError::InvalidRegionState { host, edge } => {
                write!(
                    f,
                    "sizing weight of the {edge:?} region of {host} is invalid (must be finite and strictly positive)"
                )
            }
            DockingError::FloatingIndexOutOfRange { index, entry_count } => {
                write!(
                    f,
                    "floating index {index} out of range (entry count {entry_count})"
                )
            }
            DockingError::FloatingBoundViolation { host, index } => {
                write!(
                    f,
                    "floating rect rejected: entry {index} of {host} would violate its logical bounds"
                )
            }
            DockingError::InvalidFloatingState { host, index } => {
                write!(
                    f,
                    "logical rect of floating entry {index} of {host} is invalid (must be finite, non-empty, and within the host's bounds)"
                )
            }
        }
    }
}

impl std::error::Error for DockingError {}
