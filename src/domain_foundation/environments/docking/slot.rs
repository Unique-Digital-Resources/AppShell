//! Placement slots: where a dock lives in a [`DockLayout`].
//!
//! Every dock occupies exactly one logical placement (PRD DM-07). Placements
//! are inspected with [`DockLayout::placement`](super::model::DockLayout::placement)
//! and constructed / changed only by docking operations — there is no
//! public constructor, so ad-hoc external placement cannot bypass
//! operation validation (PRD §15.3).

use std::fmt;

use super::dock::DockId;

/// An edge region of an embed host (PRD §5).
///
/// Regions communicate *where* a dock is placed on a host; the selected
/// docking mode determines *what structural operation* realizes the
/// placement (PRD §10.2–§10.3).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeRegion {
    /// The host's top edge region.
    Top,
    /// The host's bottom edge region.
    Bottom,
    /// The host's left edge region.
    Left,
    /// The host's right edge region.
    Right,
}

/// A logical placement: the slot a dock currently occupies.
///
/// Unit variant [`DockSlot::Root`] means the dock is the layout's
/// designated root and has no parent.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSlot {
    /// The dock is the designated root of the layout.
    Root,
    /// Ordered child position within a split container.
    SplitChild {
        /// The split container owning the placement.
        parent: DockId,
        /// The child index, in row/column order (left→right, top→bottom).
        index: usize,
    },
    /// Membership in a tab group.
    TabMember {
        /// The tab group owning the placement.
        parent: DockId,
        /// The member index, in tab order.
        index: usize,
    },
    /// An occupied edge region of an embed host.
    HostEdge {
        /// The host owning the placement.
        host: DockId,
        /// Which edge region is occupied.
        edge: EdgeRegion,
    },
    /// An entry in a host's floating-dock collection.
    Floating {
        /// The host owning the placement.
        host: DockId,
        /// The floating index, in floating order.
        index: usize,
    },
}

impl DockSlot {
    /// The dock that owns this slot: the parent container or host.
    /// `None` for the root slot.
    pub fn owner(&self) -> Option<DockId> {
        match self {
            DockSlot::Root => None,
            DockSlot::SplitChild { parent, .. } | DockSlot::TabMember { parent, .. } => Some(*parent),
            DockSlot::HostEdge { host, .. } | DockSlot::Floating { host, .. } => Some(*host),
        }
    }

    /// `true` when this placement is the layout root.
    pub fn is_root(&self) -> bool {
        matches!(self, DockSlot::Root)
    }
}

impl fmt::Display for DockSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DockSlot::Root => write!(f, "root"),
            DockSlot::SplitChild { parent, index } => {
                write!(f, "child {index} of {parent}")
            }
            DockSlot::TabMember { parent, index } => {
                write!(f, "tab {index} of {parent}")
            }
            DockSlot::HostEdge { host, edge } => write!(f, "{edge:?} region of {host}"),
            DockSlot::Floating { host, index } => write!(f, "floating {index} on {host}"),
        }
    }
}
