//! Floating state: the logical placement and sizing state of a host's
//! floating-dock collection (current_state/plan.md Phase 08, PRD §9.6).
//!
//! A floating entry is a dock subtree whose root is placed in a host's
//! floating collection ([`DockSlot::Floating`](super::slot::DockSlot::Floating)).
//! The collection is the PRD §7.5–§7.6 explicit placement collection —
//! floating placements are **host-anchored**: every floated structure
//! belongs to exactly one host, and the host anchors the floating layer
//! the way its edge regions anchor docked ones. There is deliberately no
//! layout-level floating collection beside it (no placement category
//! outside §7.5's list; the PRD's relationship model keeps explicit
//! placement collections on hosts). A frontend that wants a floating
//! layer without a host creates one (`CreateEmbedHost`) or promotes a
//! panel (`PromoteHost`) before floating into it — both are existing
//! operations.
//!
//! ## Position and size state (PRD §9.6 — Phase 8)
//!
//! Each floating entry carries one persistent [`FloatingRect`], stored
//! verbatim under the same representation decision as split and region
//! weights (see [`sizing`](super::sizing)) — logical state only, no
//! pixels (PRD §9.1). The state answers PRD §9.6's requirements:
//!
//! - **Sufficient to reconstruct position and dimensions** (MUST): the
//!   rect is a full placement — position *and* extent — of the entry
//!   within its host.
//! - **Which rectangle the state names** (MUST establish): the **dock
//!   body** — the dock's own logical frame. Window decoration is a
//!   frontend concern layered around it; the domain never stores or
//!   validates decoration chrome.
//! - **Constrained to the host's logical bounds** (SHOULD): the rect is
//!   stored in **fractions of the host's body** (0.0–1.0 per axis), so
//!   containment is a stored-state invariant the environment validates
//!   (`x ≥ 0`, `y ≥ 0`, `w > 0`, `h > 0`, `x + w ≤ 1`, `y + h ≤ 1`, all
//!   finite — bound endpoints inclusive). The frontend reconstructs
//!   pixel geometry by mapping the fractions onto the host's actual
//!   work area and applying the resulting geometry (PRD §9.6 SHOULD).
//!   Fraction coordinates also keep the state resolution-independent
//!   and avoid mixing a stored fraction with stored pixels (PRD §9.2's
//!   conversion-policy rule).
//! - **Structure preservation** (§7.6, §9.6 SHOULD): floating never
//!   flattens — the source detaches with its subtree intact and the
//!   entry names its root ([`FloatingEntry::dock`]).
//!
//! The rect attaches to the **slot, not the occupant** — the PRD §9.5
//! region-weight decision extended to floating: an entry replacement
//! (a survivor inheriting the slot through normalization, a tab-group
//! wrap in a floating slot) keeps the entry's rect verbatim, and a
//! detached entry's rect goes with the cleared slot (rects live with
//! the slot lifecycle — the next float starts from the baseline or the
//! caller-supplied initial rect, never the previous occupant's move
//! history).
//!
//! ## Transfer table (PRD §9.6 — mirroring the §9.2/§9.5 tables)
//!
//! | Stage | Policy |
//! |---|---|
//! | initialized | an append without an explicit rect starts at `FloatingRect::default()` — the floating baseline; the `Embed` slot can carry the drop interaction's initial rect |
//! | updated | only by `SetFloatingRect` — exactly one entry's rect is rewritten; the host's other placements and stored state stay verbatim |
//! | transferred | slot inheritance keeps the rect: a survivor that copies into an inherited floating slot keeps the entry's rect; a tab-group wrap in a floating slot carries it |
//! | cleared | removing the entry deletes its rect with the slot — the lifecycle follows the collection membership |
//!
//! Resizing / moving entries shares the established resize contracts:
//! absolute drift-free requests (each gesture frame sends the entry's
//! next **absolute** rect derived from the gesture's initial state and
//! total displacement), operation-carried minimum bounds honored
//! inclusively (PRD §9.3 SHOULD for floating docks; frontend-translated
//! fractions), and **rejection** — a conflicting request is refused,
//! never silently clamped or resolved into invalid state
//! ([`FloatingBoundViolation`](super::error::DockingError::FloatingBoundViolation)).
//! Visual stacking and z-order are **frontend-derived** (PRD §11.3);
//! the collection order is the entries' placement order, not stacking
//! state, and v1 defines no floating-rearrangement operation.

use super::dock::DockId;

/// The persistent logical placement and sizing state of a floating
/// entry (PRD §9.6): a rectangle in **fractions of the host's body** —
/// the placement and dimensions the frontend reconstructs the floating
/// dock at.
///
/// The stored rectangle names the **dock body** (the dock's own logical
/// frame; decoration is frontend-applied around it). The representation
/// invariants: `x`/`y` finite and within `[0, 1]`, `w`/`h` finite and
/// strictly positive, `x + w ≤ 1`, `y + h ≤ 1` (the entry stays within
/// its host's logical bounds — PRD §9.6 SHOULD; bound endpoints are
/// inclusive). Violating inputs are rejected by the operations; a
/// [`FloatingRect`] held in a committed layout is valid by construction,
/// and every stored rect is re-checked on import by the Phase-3 gate
/// ([`InvalidFloatingState`](super::error::DockingError::InvalidFloatingState)).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))] // absent fields restore the baseline
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingRect {
    /// The left edge of the rect, as a fraction of the host's body
    /// width (0.0 = flush left).
    pub(crate) x: f64,
    /// The top edge of the rect, as a fraction of the host's body
    /// height (0.0 = flush top).
    pub(crate) y: f64,
    /// The rect's width, as a fraction of the host's body width
    /// (strictly positive).
    pub(crate) w: f64,
    /// The rect's height, as a fraction of the host's body height
    /// (strictly positive).
    pub(crate) h: f64,
}

impl FloatingRect {
    /// The floating baseline: the rect a floating entry starts at when
    /// its append supplies none — flush top-left, half the host's body
    /// in each axis. The `EmbedSlot::Floating` default (the role the
    /// embedding weight `1.0` plays for edge regions).
    pub const BASELINE: FloatingRect = FloatingRect { x: 0.0, y: 0.0, w: 0.5, h: 0.5 };

    /// Construct a rect from host-fraction parts, rejecting values that
    /// violate the representation invariants (finite, `x`/`y` within
    /// `[0, 1]`, `w`/`h` strictly positive, `x + w ≤ 1`, `y + h ≤ 1` —
    /// endpoints inclusive).
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Option<Self> {
        let candidate = Self { x, y, w, h };
        if candidate.is_valid() {
            Some(candidate)
        } else {
            None
        }
    }

    /// `true` when the stored parts satisfy every representation
    /// invariant (the Phase-3 gate's steady-state check; non-finite
    /// values fail every ordered comparison, so the check is NaN-safe).
    pub fn is_valid(&self) -> bool {
        let parts = [self.x, self.y, self.w, self.h];
        parts.iter().all(|v| v.is_finite())
            && self.x >= 0.0
            && self.y >= 0.0
            && self.w > 0.0
            && self.h > 0.0
            && self.x + self.w <= 1.0
            && self.y + self.h <= 1.0
    }

    /// The left edge (fraction of the host's body width).
    pub fn x(&self) -> f64 {
        self.x
    }

    /// The top edge (fraction of the host's body height).
    pub fn y(&self) -> f64 {
        self.y
    }

    /// The width (fraction of the host's body width).
    pub fn w(&self) -> f64 {
        self.w
    }

    /// The height (fraction of the host's body height).
    pub fn h(&self) -> f64 {
        self.h
    }
}

impl Default for FloatingRect {
    /// The floating baseline (see [`FloatingRect::BASELINE`]).
    fn default() -> Self {
        Self::BASELINE
    }
}

/// One entry of a host's floating-dock collection: the floated subtree's
/// root dock plus the entry's persistent logical rect (PRD §7.5, §9.6).
///
/// The rect attaches to the **slot**, not the occupant (the §9.5
/// region-weight decision, see the module docs): the entry is
/// collection state, rewritten only by docking operations and carried
/// forward verbatim by slot-inheriting rewrites. The collection itself
/// stays a placement authority — the entry's `dock` is the root of the
/// floated subtree, reachable as
/// [`DockSlot::Floating { host, index }`](super::slot::DockSlot::Floating)
/// through the layout's placement scan.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingEntry {
    /// The root dock of the floated subtree.
    pub dock: DockId,
    /// The entry's persistent logical rect (PRD §9.6). Absent fields
    /// restore at the baseline on import; the stored value of a
    /// committed entry is operation-guaranteed valid.
    #[cfg_attr(feature = "serde", serde(default))]
    rect: FloatingRect,
}

impl FloatingEntry {
    /// An entry for `dock` at `rect`.
    pub(crate) fn new(dock: DockId, rect: FloatingRect) -> Self {
        Self { dock, rect }
    }

    /// The entry's persistent logical rect (the position and size the
    /// frontend reconstructs the floating dock at).
    pub fn rect(&self) -> FloatingRect {
        self.rect
    }

    /// Rewrite the entry's rect in place (the `SetFloatingRect` write —
    /// the floated dock is untouched, the collection's other entries
    /// keep their stored state verbatim). `pub(crate)`: operation
    /// territory — ad-hoc external mutation must not bypass the
    /// operation's bound validation (PRD §15.3).
    pub(crate) fn set_rect(&mut self, rect: FloatingRect) {
        self.rect = rect;
    }
}
