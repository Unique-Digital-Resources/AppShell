//! Structural operations: value-type intents for structural layout
//! changes (current_state/plan.md Phases 04–07,
//! PRD §8.1–§8.9, §8.11–§8.12, §9.2–§9.5, §9.7, OP-01…OP-04).
//!
//! One public value-type enum pair [`DockingOperation`] / [`DockingOutcome`]
//! with the one-method `apply(self, &mut model)` contract used by every
//! other environment (the [`LayersOperation`] precedent). The operation
//! value describes *what the change means*, not a sequence of model calls.
//!
//! ## Atomicity (OP-01…OP-04)
//!
//! Every operation runs on a **candidate** clone of the layout: clone,
//! mutate, normalize, validate, commit. The candidate may hold transient
//! mid-flight states exactly like those PRD DM-07 permits (rootless,
//! unplaced, below-minimum containers); the committed result must not.
//!
//! ```text
//! mutate → normalize → validate → commit
//! ```
//!
//! The mutation runs free on the candidate, the shared normalization
//! pass ([`super::normalization`]) resolves the affected rootward chain,
//! [`DockLayout::validate`](super::validation) (Phase 3) gates the
//! result, and the caller's layout is replaced only on success. A
//! failure leaves the original layout untouched — the atomicity recipe
//! fixed by the checkpoint (PRD §8.2).
//!
//! ## Undo contract (engine integration, Phase 9)
//!
//! Outcomes carry **full inverse data from day one** (Ph.1 §4.2
//! R-A1/R-A2; the [`LayersOutcome::LayerRemoved`] removed-subtree
//! precedent). Every variant exposes an [`UndoData`] via
//! [`DockingOutcome::inverse`]: the previous collection state of every
//! container the operation touched, dock payloads deleted from the layout
//! (removed subtrees in BFS order *and* cascade-destroyed containers),
//! the ids of docks created by the operation, and the previous layout
//! root. `UndoData::replay_into` is the `pub(crate)` seam the Phase-9
//! `ExecutedDockingOperation` wrapper builds `undo`/`redo` on.
//!
//! ## Frontend interaction contract (PRD §10–§11, Phase 11)
//!
//! These operation values *are* the interaction contract a
//! drag-and-drop frontend drives: a confirmed drop submits one value
//! built from intent-only inputs (source, target, region intent, carried
//! bounds — never pointer coordinates), and a rejected value is refused
//! without partial state. The stage-by-stage lifecycle mapping, the
//! region+mode tables, eligibility discovery, and the rejection payloads
//! live in the environment's module docs ([`super`], PRD §10–§11); the
//! per-operation gesture contracts below are that contract's
//! resize/move clauses.
//!
//! ## Scope and settled behaviour (Phase 4 baseline)
//!
//! - **Creation** auto-roots only from the fully empty layout
//!   (PRD §7.4). A creation that would commit an unplaced dock to a
//!   populated layout is rejected by the Phase-3 gate with
//!   [`DockingError::OrphanDock`]. Later panels arrive *placed* via
//!   [`InsertSource::NewPanel`] inside
//!   [`DockingOperation::InsertBeside`] / [`DockingOperation::TabJoin`] —
//!   no committed layout ever holds an unplaced dock, so there is no
//!   standalone "move unplaced dock" operation; dock re-placement happens
//!   through these two operations (host regions and floating targets
//!   arrive with Phases 7–8).
//! - **Normalization** (PRD §8.12). Every cleanup rule set lives in the
//!   shared pass ([`super::normalization`]): after the mutation, each
//!   operation normalizes the affected rootward chain, so a split
//!   container with fewer than two children collapses (its surviving
//!   child inherits the split's slot) and a tab group that lost its
//!   last member is removed. The whole-layout entry point
//!   `DockLayout::normalize` applies the same rules until no
//!   below-minimum container remains. The pipeline guarantees
//!   OP-03: every committed result is normalized and valid.
//! - **Sizing baseline** (the §9.2 redistribution policy is documented
//!   in [`super::normalization`]): inserting beside a target in a
//!   compatible container copies the target's weight for the new child
//!   (siblings unchanged); a newly created container starts uniform;
//!   detachment deletes the moved child's weight so the surviving
//!   children keep their relative shares; a collapsing split's parent
//!   keeps the slot's weight. Stored weights are verbatim (never
//!   rescaled — the representation decision in [`super::sizing`]).
//! - **Sizing operations** (PRD §9.2–§9.4, Phase 6):
//!   [`DockingOperation::ResizeSiblings`] rewrites the *adjacent pair*
//!   touched by the splitter gesture — the pair's total is conserved
//!   (the second region absorbs the difference), so every other child's
//!   share stays verbatim and no below-minimum container can appear
//!   (the cascade step is skipped: the operation is non-structural).
//!   The frontend drives each gesture frame with the pair-first
//!   region's **absolute** requested weight, computed from the
//!   gesture's initial state and total displacement — frames compose
//!   without accumulating rounded deltas (PRD §9.4 SHOULD).
//!   Minimum bounds (PRD §9.3) are translated by the frontend into
//!   per-operation constraints; a conflicting request is **rejected**
//!   ([`DockingError::SizeBoundViolation`], naming the first checked
//!   region that violates its bound), never clamped or silently
//!   resolved into invalid sizing. Cancelling a gesture is frontend
//!   state (Phase 11); already-applied frames roll back through the
//!   outcome's inverse data like any other operation.
//! - **Collapse** (PRD §9.7, Phase 6): [`DockingOperation::SetDockCollapsed`]
//!   toggles a panel's persistent logical collapse preference. It is
//!   presentation-meaning only: the dock keeps its placement,
//!   membership, and sizing data verbatim (the weight it holds *is*
//!   the retained restore state), so collapse never makes a dock
//!   unreachable and normalization treats collapsed members as
//!   ordinary members ([`super::normalization`]). v1 eligible category:
//!   panels only.
//! - **Tab active-index baseline** (PRD §8.7): a joined member becomes
//!   active; reordering and membership changes keep the active index
//!   pointing at the same member (the moved member carries its
//!   selection).
//! - **Occupied / edge semantics** (PRD §8.6, §8.10): dropping beside a
//!   tab member splits at the member's *group* (walking up while the
//!   resolved container is itself a tab member, so the group keeps every
//!   member); dropping beside a host-edge occupant splits that region;
//!   dropping beside a *floating* dock has no defined directional meaning
//!   in v1 and is rejected ([`DockingError::NotSplittable`]). Center
//!   drops (swap) are not operations in v1 (PRD §8.10).
//! - **Embedding** (PRD §8.8, Phase 7): [`DockingOperation::Embed`]
//!   places a source into an eligible host's edge region
//!   ([`EmbedSlot::Region`]) or appends it to the host's floating
//!   collection ([`EmbedSlot::Floating`]) — one floated subtree per
//!   entry, internal structure preserved. The host **retains its own
//!   work area**. One dock
//!   subtree per edge region: an occupied region **rejects** the embed
//!   ([`DockingError::RegionOccupied`]; the PRD §8.8 baseline — the
//!   existing occupant is never silently discarded, and v1 defines no
//!   replacement/exchange policy). Sizing baseline: a newly occupied
//!   region starts at the embedding weight `1.0` (the uniform-with-core
//!   default; the §9.5 allocation definition and the slot-vs-dock
//!   decision live in [`super::embed_host`]/[`super::sizing`]); a
//!   floating entry starts at its rect — the one carried in the slot,
//!   or the floating baseline ([`FloatingRect::default`], PRD §9.6).
//! - **Host promotion and demotion** (PRD §8.9, Phase 7): a host is
//!   either **persistent** (created via `CreateEmbedHost` or promoted
//!   by the explicit [`DockingOperation::PromoteHost`] — remains a host
//!   even when empty) or a **temporary promotion**. `Embed` auto-promotes
//!   its target panel the way §8.9 sanctions ("when a valid embedding
//!   operation requires it") — promoted solely to support that embed, so
//!   it is temporary. A temporary host whose regions and floating
//!   collection are all empty demotes back to its former category — an
//!   in-place role change that keeps identity and content — through the
//!   shared normalization pass ([`super::normalization`], the automatic
//!   clean-up side of §8.9). The panel-only collapse preference is not
//!   carried across promotion or demotion (v1: hosts are not a
//!   collapsible category, so a promoted/demoted dock lands expanded).
//! - **Region resizing** (PRD §9.5, Phase 7):
//!   [`DockingOperation::ResizeRegion`] rewrites exactly one occupied
//!   region's stored weight — every other placement and stored weight of
//!   the host stays verbatim (each edge resizes against its inner
//!   boundary; the allocation model is defined in [`super::embed_host`]).
//!   The gesture and constraint contracts mirror `ResizeSiblings`:
//!   absolute drift-free requests, operation-carried minimums, rejection
//!   (never clamping), and the finite/strictly-positive invariant as the
//!   implicit bound. Region weights are written whole (no vector
//!   rebuild), so a resize needs no stored-garbage funnel — imported
//!   garbage region weights are caught by the Phase-3 gate
//!   (`InvalidRegionState`) like any steady-state violation.
//! - **Removal of embedded structures** (PRD §8.11): `Remove` of a host
//!   removes its **whole arrangement** — the dock, every occupied-region
//!   subtree, and every floating entry (the BFS subtree crossings include
//!   host edges) — with the region weights going with the cleared slots.
//!   `Remove` of an occupant detaches it from its region/floating slot
//!   alone and may demote the promoted host it emptied; the host
//!   application keeps content lifetime either way.
//! - **Floating state** (PRD §9.6, Phase 8; the representation and
//!   lifecycle docs live in [`super::floating`]): floating placements
//!   are **host-anchored** — the per-host floating collection is the
//!   PRD §7.5–§7.6 explicit placement collection, floated roots are
//!   explicit entries in it, and there is deliberately no layout-level
//!   floating collection beside it. `Embed` detaches a source (panel,
//!   tab group, or any subtree — floating never flattens) and appends
//!   it as one entry carrying its initial [`FloatingRect`]; re-docking
//!   a floater back is the same ops in reverse (floater as source into
//!   an `Embed` region, `InsertBeside`, or `TabJoin` target — the
//!   vacated entry's rect is cleared with the slot lifetime; a group
//!   wrapped in a floating slot inherits the entry's rect). A floating
//!   dock as an `InsertBeside`/directional **target** stays rejected
//!   (`NotSplittable` — no directional layout surface; PRD §8.6
//!   baseline). [`DockingOperation::SetFloatingRect`] rewrites exactly
//!   one entry's rect (absolute drift-free requests, carried minimum
//!   bounds, rejection policy — never clamped); z-order/stacking is
//!   frontend-derived (PRD §11.3), so no v1 floating-rearrangement
//!   operation exists.
//!
//! [`LayersOperation`]: crate::domain_foundation::environments::layers::operations::LayersOperation
//! [`LayersOutcome::LayerRemoved`]:
//! crate::domain_foundation::environments::layers::operations::LayersOutcome::LayerRemoved

use std::fmt;

use super::embed_host::{HostRegions, REGION_WEIGHT_BASELINE};
use super::dock::{ContentRef, Dock, DockId, DockKind, DockRole, SplitOrientation};
use super::error::DockingError;
use super::floating::{FloatingEntry, FloatingRect};
use super::model::DockLayout;
use super::normalization::{collapse_cascade, prune_weight};
use super::sizing::SplitSizes;
use super::slot::{DockSlot, EdgeRegion};

/// The direction an insertion is made along (PRD §8.6).
///
/// `Left`/`Right` imply a row container, `Top`/`Bottom` a column
/// container; see [`SplitDirection::orientation`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitDirection {
    /// Insert to the left of the target (row order).
    Left,
    /// Insert to the right of the target (row order).
    Right,
    /// Insert above the target (column order).
    Top,
    /// Insert below the target (column order).
    Bottom,
}

impl SplitDirection {
    /// The split orientation the direction implies.
    pub fn orientation(self) -> SplitOrientation {
        match self {
            SplitDirection::Left | SplitDirection::Right => SplitOrientation::Row,
            SplitDirection::Top | SplitDirection::Bottom => SplitOrientation::Column,
        }
    }

    /// `true` when the source lands *after* the target in child order
    /// (`Right`, `Bottom`); `false` when it lands before (`Left`, `Top`).
    pub fn inserts_after(self) -> bool {
        matches!(self, SplitDirection::Right | SplitDirection::Bottom)
    }
}

impl fmt::Display for SplitDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitDirection::Left => write!(f, "left"),
            SplitDirection::Right => write!(f, "right"),
            SplitDirection::Top => write!(f, "top"),
            SplitDirection::Bottom => write!(f, "bottom"),
        }
    }
}

/// The slot of a host an [`DockingOperation::Embed`] places its source
/// into (PRD §8.8, §9.6).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmbedSlot {
    /// Occupy an edge region of the host (one dock subtree per region;
    /// occupied regions reject the embed).
    Region(EdgeRegion),
    /// Append to the host's floating-dock collection at the end (the
    /// collection order is placement order; visual stacking is
    /// frontend-derived, PRD §11.3). The entry carries the given
    /// initial logical rect (PRD §9.6 — the drop interaction supplies
    /// it); `None` starts at the floating baseline
    /// ([`FloatingRect::BASELINE`](super::floating::FloatingRect::BASELINE)).
    Floating {
        /// The entry's initial logical rect, or `None` for the
        /// baseline.
        rect: Option<FloatingRect>,
    },
}

/// The source of an insertion: an existing dock (moved from its current
/// placement, identity preserved) or a panel created for this insertion.
///
/// The created-panel variant is how additional panels enter a populated
/// layout: the dock is created inside the operation's candidate and
/// placed immediately. A failed insertion discards the candidate
/// together with the created dock (OP-01: no orphan residue).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum InsertSource {
    /// Move an existing dock into the requested placement.
    Existing(DockId),
    /// Create a new panel dock (with the given [`ContentRef`]) and
    /// place it.
    NewPanel {
        /// The host-supplied content reference of the new panel.
        content: Option<ContentRef>,
    },
}

/// A docking operation: a value describing one structural change
/// (PRD §8.1).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DockingOperation {
    /// Create a panel dock.
    ///
    /// On the fully empty layout the panel is auto-rooted (PRD §7.4). On
    /// a populated layout the created panel would hold no placement; the
    /// Phase-3 gate rejects the candidate with
    /// [`DockingError::OrphanDock`]. Panels created *into* a layout use
    /// [`InsertSource::NewPanel`] within
    /// [`DockingOperation::InsertBeside`] / [`DockingOperation::TabJoin`].
    CreatePanel {
        /// The host-supplied content reference.
        content: Option<ContentRef>,
    },
    /// Create an embed host dock.
    ///
    /// Root rule as [`DockingOperation::CreatePanel`]; the host starts
    /// with empty edge regions and an empty floating collection.
    CreateEmbedHost {
        /// The content reference of the host's own work area.
        content: Option<ContentRef>,
    },
    /// Place `source` beside `target` along `direction`.
    ///
    /// When the resolved target sits in a compatible container (its
    /// split orientation matches the direction), the source is inserted
    /// into that container; otherwise a new container is created around
    /// the target and the source, inheriting the target's slot
    /// (PRD §8.6).
    InsertBeside {
        /// The dock being placed (existing or newly created).
        source: InsertSource,
        /// The dock the source is placed beside.
        target: DockId,
        /// The direction of the insertion.
        direction: SplitDirection,
    },
    /// Make `source` a member of the tab group holding `target` — or,
    /// when `target` is in no group, create a tab group around both that
    /// inherits the target's slot. The joined dock becomes the active
    /// member; a dock already inside the target's group is a no-op
    /// success (PRD §8.7). v1 membership is panels only.
    TabJoin {
        /// The dock being placed (existing or newly created).
        source: InsertSource,
        /// The panel the source joins.
        target: DockId,
    },
    /// Reorder a member of a tab group. Convention follows the Layers
    /// precedent: `to` is the insertion index in the member list with
    /// the member removed (validated against the pre-removal count).
    ReorderTab {
        /// The tab group.
        group: DockId,
        /// The member's current index.
        from: usize,
        /// The insertion index for the member.
        to: usize,
    },
    /// Select the active member of a tab group (index into `members`).
    SetActiveTab {
        /// The tab group.
        group: DockId,
        /// The member index to activate.
        index: usize,
    },
    /// Remove a dock and its entire descendant subtree (PRD §8.11
    /// semantics (b); a panel is a leaf, so removing a panel removes the
    /// panel alone). Content lifetime stays with the host application;
    /// the removal is undoable via the outcome payload.
    Remove {
        /// The dock to remove.
        dock: DockId,
    },
    /// Resize the **adjacent pair** of split children at `first` and
    /// `first + 1` of `container` — the two regions a splitter gesture
    /// sits between (PRD §9.4).
    ///
    /// The pair's first region's weight becomes `requested` and the
    /// second region absorbs the difference, so the pair's total — and
    /// therefore every other child's share — is conserved (PRD §9.2:
    /// verbatim weights elsewhere).
    ///
    /// Gesture contract: the frontend measures pointer movement and
    /// sends, per frame, the pair-first region's **absolute** requested
    /// weight derived from the gesture's initial state and the total
    /// displacement so far (PRD §9.4 SHOULD) — frames therefore compose
    /// without accumulating rounded deltas. Cancelling the gesture is
    /// frontend interaction state; frames already applied are rolled
    /// back through the outcome's inverse data (`UndoData`), like any
    /// other operation.
    ///
    /// Minimum bounds (`first_min` / `second_min`, logical weights in
    /// the pair's units — PRD §9.3) are translated by the frontend from
    /// its available geometry. A request would violate a bound (or the
    /// sizing invariant: weights must be finite and strictly positive)
    /// is rejected — the environment's defined constraint policy
    /// ([`DockingError::SizeBoundViolation`]; never silently clamped,
    /// never invalid sizing stored). A bound endpoint itself is honored
    /// inclusively.
    ResizeSiblings {
        /// The split container holding the pair.
        container: DockId,
        /// The child index where the pair starts; the pair is
        /// `first` and `first + 1` in child (weight) order.
        first: usize,
        /// The requested post-gesture weight of the pair's first region.
        requested: f64,
        /// The pair's first region's minimum logical weight
        /// (`None` = unconstrained).
        first_min: Option<f64>,
        /// The pair's second region's minimum logical weight
        /// (`None` = unconstrained).
        second_min: Option<f64>,
    },
    /// Set a panel dock's logical collapse preference
    /// (PRD §9.7). The dock keeps its placement, membership, and sizing
    /// data verbatim — the preference is presentation meaning for the
    /// frontend only, so a collapsed dock stays placed, traversable,
    /// valid, and restorable (restoring is the flag clearing). v1
    /// eligible category: panels; a set that matches the dock's current
    /// preference is a no-op success (the [`ReorderTab`](DockingOperation::ReorderTab)
    /// `from == to` precedent).
    SetDockCollapsed {
        /// The panel dock.
        dock: DockId,
        /// `true` collapses the dock's presentation; `false` restores it.
        collapsed: bool,
    },
    /// Embed `source` into `host`'s edge region or floating collection
    /// (PRD §8.8, §9.6).
    ///
    /// The host retains its own work area — only its regions / floating
    /// collection change. `host` may name a **panel**, which is then
    /// auto-promoted to a temporary host (PRD §8.9 promotion: identity
    /// and content preserved; regions/floating start empty); a named
    /// split or tab group is rejected
    /// ([`DockingError::IneligibleDock`]) — structural containers can
    /// neither host nor promote. An occupied edge region rejects the
    /// embed ([`DockingError::RegionOccupied`], the PRD §8.8 baseline);
    /// floating accepts any number of entries, appended last.
    ///
    /// Sizing: a freshly occupied region starts at the embedding weight
    /// `1.0` (the §9.5 baseline; see [`super::embed_host`]). A floating
    /// entry carries the slot's initial [`FloatingRect`] — the drop
    /// interaction's logical placement — or the floating baseline when
    /// the rect is `None`; an initial rect that violates the §9.6
    /// bounds rejects ([`DockingError::FloatingBoundViolation`]).
    /// Floating preserves the source's internal structure (§7.6: never
    /// flattened); the source may be a panel, a tab group, or any other
    /// eligible subtree (the §8.11 "float from tab group" interaction is
    /// the member detach). The source detaches from its former slot
    /// like any move (cascade
    /// normalization follows; demotion of a thereby-emptied temporary
    /// host is part of that pass).
    Embed {
        /// The dock being embedded (existing or newly created panel).
        source: InsertSource,
        /// The hosting dock (an embed host, or a panel to auto-promote).
        host: DockId,
        /// Which host slot receives the source.
        slot: EmbedSlot,
    },
    /// Promote a panel to embed-host status (PRD §8.9). The dock keeps
    /// its identity and content (its work area becomes the host's
    /// underlying area); regions and the floating collection start
    /// empty. An explicit promotion is an **intentionally persistent**
    /// host — it remains a host even when empty (the PRD §8.9
    /// distinction: `Embed`'s auto-promotion is the temporary kind,
    /// demoted by normalization once its last embedded/floating dock
    /// leaves). The panel's collapse preference does not carry over
    /// (v1: hosts are not a collapsible category). Promoting a dock
    /// that is already a host — persistent or temporary — is a no-op
    /// success. Splits and tab groups are rejected
    /// ([`DockingError::IneligibleDock`]): they hold no work area to
    /// preserve.
    PromoteHost {
        /// The panel to promote.
        dock: DockId,
    },
    /// Resize one occupied edge region of a host: the region's stored
    /// weight becomes `requested` (PRD §9.5).
    ///
    /// Each region resizes against its inner boundary — the boundary
    /// between the region and the host's content — so exactly **one**
    /// stored weight is rewritten; the host's other placements and
    /// stored weights stay verbatim, and the top/bottom ↔ left/right
    /// interaction is the defined two-stage allocation
    /// ([`super::embed_host`], PRD §9.5) rather than a rewrite of
    /// neighbours.
    ///
    /// Gesture contract (as `ResizeSiblings`, PRD §9.4): the frontend
    /// translates the boundary gesture into the region's **absolute**
    /// requested weight derived from the gesture's initial proportions
    /// and the total displacement — frames compose drift-free.
    /// Cancellation is frontend state; applied frames roll back through
    /// the outcome's inverse data.
    ///
    /// Constraint policy (as `ResizeSiblings`, PRD §9.3): rejection.
    /// `min` is the region's minimum logical weight translated by the
    /// frontend; the finite/strictly-positive invariant is the implicit
    /// bound; bound endpoints are honored inclusively
    /// ([`DockingError::RegionBoundViolation`]). v1 carries one-sided
    /// (minimum) bounds; a content-side minimum expresses as the
    /// region's maximum — a MAY the v1 interval math does not include.
    /// Only occupied regions resize
    /// ([`DockingError::RegionUnoccupied`]); the write covers the whole
    /// region weight, so no stored-garbage rebuild can fail — imported
    /// garbage is caught by the Phase-3 gate.
    ResizeRegion {
        /// The embed host.
        host: DockId,
        /// The region to resize (occupied).
        edge: EdgeRegion,
        /// The requested post-gesture weight of the region.
        requested: f64,
        /// The region's minimum logical weight (`None` = unconstrained).
        min: Option<f64>,
    },
    /// Set one floating entry's persistent logical rect (PRD §9.6,
    /// Phase 8): the entry at `index` of `host`'s floating collection
    /// takes `rect` — position *and* dimensions in one write, so a
    /// move, a resize, or a combined drag gesture each land in one
    /// operation.
    ///
    /// The rect is [`FloatingRect`] host-fraction state (the PRD §9.6
    /// decided representation: the **dock body** rectangle, contained
    /// in the host's logical bounds; the frontend reconstructs pixel
    /// geometry from it).
    ///
    /// Gesture contract (as `ResizeSiblings` / `ResizeRegion`, PRD
    /// §9.4): the frontend measures the drag and sends, per frame, the
    /// entry's **absolute** requested rect derived from the gesture's
    /// initial state and total displacement — frames compose
    /// drift-free. Cancellation is frontend state; applied frames roll
    /// back through the outcome's inverse data.
    ///
    /// Constraint policy (as `ResizeRegion`, PRD §9.3): rejection.
    /// `min_w` / `min_h` are the entry's minimum logical widths /
    /// heights translated by the frontend (the floating-dock minimums
    /// §9.3 SHOULD); the rect invariant (finite, `x`/`y` ≥ 0,
    /// `w`/`h` > 0, within the host's bounds — `x + w ≤ 1`,
    /// `y + h ≤ 1`) is always the implicit bound; bound endpoints are
    /// honored inclusively
    /// ([`DockingError::FloatingBoundViolation`]); never clamped.
    /// The write is non-structural: the floated dock, its placement,
    /// and every other entry of the collection stay verbatim (no
    /// normalization step runs — no membership changes).
    SetFloatingRect {
        /// The embed host anchoring the floating entry.
        host: DockId,
        /// The entry to rewrite (its index in the floating
        /// collection).
        index: usize,
        /// The requested post-gesture logical rect.
        rect: FloatingRect,
        /// The entry's minimum logical width fraction
        /// (`None` = unconstrained).
        min_w: Option<f64>,
        /// The entry's minimum logical height fraction
        /// (`None` = unconstrained).
        min_h: Option<f64>,
    },
}

/// The outcome of a successful [`DockingOperation`].
///
/// Every variant carries the inverse of the operation via
/// [`DockingOutcome::inverse`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DockingOutcome {
    /// A dock was created. `is_root` is `true` when the creation
    /// auto-rooted it (the layout was empty beforehand).
    Created {
        /// The new dock's id.
        dock: DockId,
        /// Whether the new dock became the layout root.
        is_root: bool,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A dock was placed beside another along a split direction.
    /// `container` is the split now owning both (the reused or the newly
    /// created container).
    InsertedBeside {
        /// The placed dock.
        source: DockId,
        /// The dock it was placed beside.
        target: DockId,
        /// The split container now owning both docks.
        container: DockId,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A dock joined a tab group (or a group was created around it and
    /// the target). `group` is the group after the operation; the joined
    /// dock is its active member.
    TabJoined {
        /// The placed dock.
        source: DockId,
        /// The dock it joined.
        target: DockId,
        /// The tab group after the operation.
        group: DockId,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A tab member was reordered.
    TabReordered {
        /// The tab group.
        group: DockId,
        /// The member's previous index.
        from: usize,
        /// The member's new (insertion) index.
        to: usize,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A tab group's active member changed.
    ActiveTabSelected {
        /// The tab group.
        group: DockId,
        /// The now-active member index.
        active: usize,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A dock and its descendant subtree were removed. `docks` lists the
    /// removed docks in BFS order (the removed dock first, then its
    /// descendants) for re-insertion; the inverse additionally holds
    /// cascade-destroyed containers and every container's previous state.
    Removed {
        /// The removed docks, BFS order (dock first, then descendants).
        docks: Vec<Dock>,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// An adjacent split-child pair was resized: child `first`'s weight
    /// became the requested value and the pair's second region absorbed
    /// the difference. `previous` holds the pair's weights before the
    /// change, in child order (the `CanvasOutcome::Moved` precedent of
    /// carrying the replaced value on the outcome); the full inverse is
    /// in `inverse`.
    SiblingsResized {
        /// The resized split container.
        container: DockId,
        /// The pair's first child index.
        first: usize,
        /// The pair's previous weights, in child order.
        previous: [f64; 2],
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A panel's logical collapse preference was set (restored docks
    /// keep their placement and allocation verbatim).
    /// `previous` is the preference before the change; a set that
    /// matched the current preference reports it unchanged.
    DockCollapsedSet {
        /// The panel dock.
        dock: DockId,
        /// The preference after the operation.
        collapsed: bool,
        /// The preference before the operation.
        previous: bool,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A dock was embedded into a host slot. `promoted` is `true` when
    /// the operation auto-promoted the target panel to host status
    /// (PRD §8.9) — its pre-promotion panel state (including the
    /// collapse preference) is in the inverse data.
    Embedded {
        /// The embedded dock.
        source: DockId,
        /// The host dock (after any promotion).
        host: DockId,
        /// The slot the source occupies.
        slot: EmbedSlot,
        /// Whether the target was promoted by this operation.
        promoted: bool,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A panel was promoted to embed-host status (identity and content
    /// preserved; a temporary promotion, PRD §8.9). A no-op promotion
    /// of an already-host dock reports `promoted: false`.
    HostPromoted {
        /// The promoted dock.
        dock: DockId,
        /// `false` for the no-op success on an already-host dock.
        promoted: bool,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// An occupied edge region was resized: the region's weight became
    /// the requested value with every other stored weight of the host
    /// verbatim. `previous` holds the weight before the change (the
    /// `SiblingsResized` precedent); the full inverse is in `inverse`.
    RegionResized {
        /// The resized host.
        host: DockId,
        /// The resized edge region.
        edge: EdgeRegion,
        /// The region's weight before the change.
        previous: f64,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
    /// A floating entry's logical rect was set: the entry at `index`
    /// took the requested rect with every other placement and stored
    /// state of the host verbatim. `previous` holds the rect before the
    /// change (the `RegionResized` precedent); the full inverse is in
    /// `inverse`.
    FloatingRectSet {
        /// The host anchoring the floating entry.
        host: DockId,
        /// The rewritten entry's index.
        index: usize,
        /// The entry's rect before the change.
        previous: FloatingRect,
        /// Undo/redo inverse data.
        inverse: UndoData,
    },
}

impl DockingOutcome {
    /// The inverse data carried by whatever variant this is (engine undo
    /// reads outcomes uniformly — Phase 9).
    pub fn inverse(&self) -> &UndoData {
        match self {
            DockingOutcome::Created { inverse, .. }
            | DockingOutcome::InsertedBeside { inverse, .. }
            | DockingOutcome::TabJoined { inverse, .. }
            | DockingOutcome::TabReordered { inverse, .. }
            | DockingOutcome::ActiveTabSelected { inverse, .. }
            | DockingOutcome::Removed { inverse, .. }
            | DockingOutcome::SiblingsResized { inverse, .. }
            | DockingOutcome::DockCollapsedSet { inverse, .. }
            | DockingOutcome::Embedded { inverse, .. }
            | DockingOutcome::HostPromoted { inverse, .. }
            | DockingOutcome::RegionResized { inverse, .. }
            | DockingOutcome::FloatingRectSet { inverse, .. } => inverse,
        }
    }
}

/// Inverse data carried by every [`DockingOutcome`]: the state the
/// operation replaced, recorded so it can be replayed to restore the
/// pre-operation layout exactly (engine undo, Phase 9).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UndoData {
    /// Previous collection state of every container — and, since
    /// Phases 6–7, the collapse preference of every panel and the
    /// category payload of every promoted/demoted host — the operation
    /// mutated, in mutation order. `UndoData::replay_into` restores
    /// them in reverse.
    pub containers: Vec<FormerCollection>,
    /// Dock payloads deleted from the layout by the operation: removed
    /// subtrees and cascade-destroyed containers, in deletion order.
    /// Replayed (re-inserted) before `containers`.
    pub removed: Vec<Dock>,
    /// Docks created by the operation (removed again on undo).
    pub created: Vec<DockId>,
    /// The layout root before the operation, restored verbatim —
    /// idempotent for operations that did not change the root.
    pub previous_root: Option<DockId>,
}

impl UndoData {
    /// The inverse of a no-op (nothing changed).
    fn identity(layout: &DockLayout) -> Self {
        Self { previous_root: layout.root(), ..Default::default() }
    }

    /// Replay the inverse onto `layout`, restoring the pre-operation
    /// state. `pub(crate)`: the seam for engine-side undo (Phase 9).
    ///
    /// The order is load-bearing: dock payloads exist again *before*
    /// container records reference them, records restore newest-first so
    /// each snapshot overwrites the later mutations of its container,
    /// and the root is restored before created docks are deleted.
    pub(crate) fn replay_into(self, layout: &mut DockLayout) {
        for dock in self.removed {
            layout.put_dock(dock);
        }
        for record in self.containers.iter().rev() {
            layout.restore_collection(record);
        }
        layout.set_root_unchecked(self.previous_root);
        for id in self.created {
            layout.take_dock(id);
        }
    }
}

/// The previous state of one container collection — or, since Phase 6,
/// of the panel collapse preference — that an operation replaced
/// (PRD §8.1 item 8: effects on placement, sizing, and selection).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum FormerCollection {
    /// A split container's children and relative sizes, verbatim.
    Split {
        /// The split container.
        id: DockId,
        /// Children in order before the change.
        children: Vec<DockId>,
        /// Relative sizes before the change.
        sizes: SplitSizes,
    },
    /// A tab group's members and active index, verbatim.
    Tab {
        /// The tab group.
        id: DockId,
        /// Members in tab order before the change.
        members: Vec<DockId>,
        /// Active member index before the change.
        active: usize,
    },
    /// A host edge region's previous occupant and its sizing weight
    /// (PRD §9.5 — occupant and preference restore together; the
    /// `assign` clearing path handles the weight lifecycle).
    Region {
        /// The embed host.
        host: DockId,
        /// Which edge region.
        edge: EdgeRegion,
        /// The occupant before the change (`None` = empty region).
        previous: Option<DockId>,
        /// The occupant's sizing weight before the change (`None` when
        /// the region was empty — the replay resets the cleared slot to
        /// the embedding baseline through `assign`).
        weight: Option<f64>,
    },
    /// A host's floating-dock collection, verbatim — full entries (the
    /// floated dock *and* the entry's logical rect, restored together —
    /// the [`Region`](FormerCollection::Region) record's
    /// occupant-and-preference precedent, PRD §9.6).
    Floating {
        /// The embed host.
        host: DockId,
        /// Floating entries in order before the change.
        entries: Vec<FloatingEntry>,
    },
    /// A panel's logical collapse preference (PRD §9.7), recorded when
    /// a [`SetDockCollapsed`](DockingOperation::SetDockCollapsed)
    /// operation flips it. Not a structural collection: the payload
    /// record completes the inverse coverage of the one dock field the
    /// operation value-changed.
    Panel {
        /// The panel dock.
        id: DockId,
        /// The collapse preference before the change.
        collapsed: bool,
    },
    /// A panel's pre-promotion category payload (PRD §8.9), recorded
    /// when `PromoteHost` / `Embed` turns the panel into a host.
    /// Identity and content survive promotion verbatim (the replay
    /// reads them off the promoted host), so the record carries the
    /// panel-only collapse preference the promotion dropped.
    Promotion {
        /// The promoted panel dock.
        id: DockId,
        /// The panel's collapse preference before the promotion.
        collapsed: bool,
    },
    /// A host's pre-demotion role (PRD §8.9), recorded when the
    /// normalization pass demotes an emptied temporary host back to a
    /// panel. Demotion fires only on hosts whose regions and floating
    /// collection are empty, so the replay rebuilds the fresh-empty
    /// temporary host exactly (`Demotion` has no payload beyond the id;
    /// content flows through unchanged).
    Demotion {
        /// The demoted host dock.
        id: DockId,
    },
}

/// Resolve an id up the tab-member chain: while `id` occupies a tab
/// member slot, its group is the insertion-resolved target, so an
/// insertion always splits at the outermost tab boundary and the groups
/// keep every member. The visited guard terminates on exotic (invalid)
/// cyclic states; such states are rejected by the Phase-3 gate anyway.
fn resolve_insertion_target(layout: &DockLayout, id: DockId) -> DockId {
    let mut resolved = id;
    let mut visited: Vec<DockId> = Vec::new();
    while let Some(DockSlot::TabMember { parent, .. }) = layout.placement(resolved) {
        if !visited.contains(&parent) {
            visited.push(resolved);
            resolved = parent;
        } else {
            break;
        }
    }
    resolved
}

impl DockingOperation {
    /// Apply the operation to `layout` (PRD §8.1, OP-01…OP-04).
    ///
    /// Canonical pipeline (module docs): the operation runs on a
    /// candidate clone (mutate → normalize — the shared pass), the
    /// Phase‑3 gate validates the committed result, and the candidate is
    /// swapped in only on success; on failure `layout` is untouched and
    /// the first violated rule is returned.
    pub fn apply(self, layout: &mut DockLayout) -> Result<DockingOutcome, DockingError> {
        let mut candidate = layout.clone();
        let outcome = self.apply_on_candidate(&mut candidate)?;
        // Phase-3 gate: only a whole-layout-valid candidate commits
        // (OP-02). Failure discards the candidate; `layout` is unchanged.
        candidate.validate()?;
        *layout = candidate;
        Ok(outcome)
    }

    /// Dispatch on the candidate. Returns the outcome; the candidate may
    /// hold transient (mid-operation) states at this point.
    fn apply_on_candidate(self, layout: &mut DockLayout) -> Result<DockingOutcome, DockingError> {
        match self {
            DockingOperation::CreatePanel { content } => {
                let mut undo = UndoData::identity(layout);
                let id = layout.create_panel(content);
                undo.created.push(id);
                Ok(DockingOutcome::Created {
                    dock: id,
                    is_root: layout.root() == Some(id),
                    inverse: undo,
                })
            }
            DockingOperation::CreateEmbedHost { content } => {
                let mut undo = UndoData::identity(layout);
                let id = layout.create_embed_host(content);
                undo.created.push(id);
                Ok(DockingOutcome::Created {
                    dock: id,
                    is_root: layout.root() == Some(id),
                    inverse: undo,
                })
            }
            DockingOperation::InsertBeside { source, target, direction } => {
                apply_insert_beside(layout, source, target, direction)
            }
            DockingOperation::TabJoin { source, target } => apply_tab_join(layout, source, target),
            DockingOperation::ReorderTab { group, from, to } => {
                apply_reorder_tab(layout, group, from, to)
            }
            DockingOperation::SetActiveTab { group, index } => {
                apply_set_active_tab(layout, group, index)
            }
            DockingOperation::Remove { dock } => apply_remove(layout, dock),
            DockingOperation::ResizeSiblings {
                container,
                first,
                requested,
                first_min,
                second_min,
            } => {
                apply_resize_siblings(layout, container, first, requested, first_min, second_min)
            }
            DockingOperation::SetDockCollapsed { dock, collapsed } => {
                apply_set_dock_collapsed(layout, dock, collapsed)
            }
            DockingOperation::Embed { source, host, slot } => {
                apply_embed(layout, source, host, slot)
            }
            DockingOperation::PromoteHost { dock } => apply_promote_host(layout, dock),
            DockingOperation::ResizeRegion { host, edge, requested, min } => {
                apply_resize_region(layout, host, edge, requested, min)
            }
            DockingOperation::SetFloatingRect { host, index, rect, min_w, min_h } => {
                apply_set_floating_rect(layout, host, index, rect, min_w, min_h)
            }
        }
    }
}

// --- Detach and cascade helpers ---

/// Detach `id` from whatever slot it occupies, recording the former
/// parent's collection state (PRD §8.5). Returns the dock that owned the
/// vacated slot (`None` for the Root slot — the root has no parent
/// collection; its pointer change is covered by `previous_root`).
///
/// The former parent may drop below its child minimum; the caller runs
/// [`collapse_cascade`] (the shared normalization pass,
/// [`super::normalization`]) *after* the insertion step so the
/// destination can never be caught mid-build by the cascade.
fn detach_dock(
    layout: &mut DockLayout,
    undo: &mut UndoData,
    id: DockId,
) -> Result<Option<DockId>, DockingError> {
    match layout.placement(id) {
        None => Err(DockingError::OrphanDock { id }),
        Some(DockSlot::Root) => {
            layout.set_root_unchecked(None);
            Ok(None)
        }
        Some(DockSlot::SplitChild { parent, index }) => {
            record_split(layout, undo, parent);
            if let Some(dock) = layout.get_mut(parent) {
                if let DockRole::Split { children, sizes, .. } = &mut dock.role {
                    children.remove(index);
                    *sizes = prune_weight(sizes, index, parent)?;
                }
            }
            Ok(Some(parent))
        }
        Some(DockSlot::TabMember { parent, index }) => {
            record_tab(layout, undo, parent);
            if let Some(dock) = layout.get_mut(parent) {
                if let DockRole::TabGroup { members, active, .. } = &mut dock.role {
                    members.remove(index);
                    if index < *active {
                        *active -= 1;
                    } else if index == *active {
                        // The active member left: keep the selection at
                        // the same position, clamped to the last member
                        // (PRD §8.7 active-selection validity).
                        *active = (*active).min(members.len().saturating_sub(1));
                    }
                }
            }
            Ok(Some(parent))
        }
        Some(DockSlot::HostEdge { host, edge }) => {
            record_region(layout, undo, host, edge);
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                    // Clearing resets the slot's weight to the embed
                    // baseline (weights live with the region lifecycle).
                    regions.assign(edge, None, REGION_WEIGHT_BASELINE);
                }
            }
            Ok(Some(host))
        }
        Some(DockSlot::Floating { host, index }) => {
            record_floating(layout, undo, host);
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                    floating.remove(index);
                }
            }
            Ok(Some(host))
        }
    }
}

/// Record a split container's current children and sizes (immutable
/// snapshot) before a mutation changes them. Shared with the shared
/// normalization pass ([`super::normalization`]) for undo bookkeeping.
pub(crate) fn record_split(layout: &DockLayout, undo: &mut UndoData, id: DockId) {
    if let Some(DockRole::Split { children, sizes, .. }) =
        layout.get(id).map(|dock| &dock.role)
    {
        undo.containers.push(FormerCollection::Split {
            id,
            children: children.clone(),
            sizes: sizes.clone(),
        });
    }
}

/// Record a tab group's current members and active index (shared with
/// the normalization pass; see `record_split`).
pub(crate) fn record_tab(layout: &DockLayout, undo: &mut UndoData, id: DockId) {
    if let Some(DockRole::TabGroup { members, active, .. }) =
        layout.get(id).map(|dock| &dock.role)
    {
        undo.containers.push(FormerCollection::Tab {
            id,
            members: members.clone(),
            active: *active,
        });
    }
}

/// Record a host edge region's current occupant and its sizing weight
/// (shared with the normalization pass; see `record_split`).
pub(crate) fn record_region(layout: &DockLayout, undo: &mut UndoData, host: DockId, edge: EdgeRegion) {
    let (previous, weight) = layout
        .get(host)
        .and_then(|dock| dock.host_regions())
        .map(|regions| (regions.on(edge), regions.weight_on(edge)))
        .unwrap_or((None, None));
    undo.containers.push(FormerCollection::Region { host, edge, previous, weight });
}

/// Record a host's current floating collection (shared with the
/// normalization pass; see `record_split`).
pub(crate) fn record_floating(layout: &DockLayout, undo: &mut UndoData, host: DockId) {
    let entries = layout
        .get(host)
        .and_then(|dock| dock.floating())
        .map(|floating| floating.to_vec());
    if let Some(entries) = entries {
        undo.containers.push(FormerCollection::Floating { host, entries });
    }
}

/// Record a panel's pre-promotion payload (the collapse preference the
/// promotion drops; identity and content survive, PRD §8.9). Shared by
/// `PromoteHost` and `Embed`'s auto-promotion.
pub(crate) fn record_promotion(layout: &DockLayout, undo: &mut UndoData, id: DockId) {
    let collapsed = layout
        .get(id)
        .and_then(|dock| dock.collapsed())
        .unwrap_or(false);
    undo.containers.push(FormerCollection::Promotion { id, collapsed });
}

/// Record a host's pre-demotion role (the normalization demotion rule;
/// see `record_region`).
pub(crate) fn record_demotion(undo: &mut UndoData, id: DockId) {
    undo.containers.push(FormerCollection::Demotion { id });
}

// --- Operations ---

/// `InsertBeside` (PRD §8.6). Preconditions, in check order: the target
/// exists; for an existing source — it exists, and the resolved target is
/// not the source or inside the source's subtree (a cycle: the new
/// container would sit inside the source itself). Postconditions: the
/// source occupies a split-child slot of `container` (reused compatible
/// container or a fresh one inheriting the target's slot); the former
/// parent is left valid via the collapse cascade; failure leaves the
/// layout untouched.
fn apply_insert_beside(
    layout: &mut DockLayout,
    source: InsertSource,
    target: DockId,
    direction: SplitDirection,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    // 1. Target existence (a named argument that must exist).
    if !layout.contains(target) {
        return Err(DockingError::DockNotFound { id: target });
    }
    // 2. Resolve the insertion target up the tab-member chain, then
    //    reject cycles for an existing source (self-include: a dock is
    //    its own descendant in the Layers precedent).
    let resolved = resolve_insertion_target(layout, target);
    let is_existing = matches!(source, InsertSource::Existing(_));
    let source_id = match source {
        InsertSource::Existing(id) => {
            if !layout.contains(id) {
                return Err(DockingError::DockNotFound { id });
            }
            if layout.is_descendant(resolved, id) {
                return Err(DockingError::CycleDetected { id });
            }
            id
        }
        InsertSource::NewPanel { content } => {
            // The target exists, so the layout is non-empty and the
            // created panel cannot auto-root. Created docks are removed
            // again on undo (the operation owns their lifetime).
            let id = layout.create_panel(content);
            undo.created.push(id);
            id
        }
    };

    // 3. Detach the source (records its former parent). A new panel has
    //    nothing to detach.
    let mut cascade_start = None;
    if is_existing {
        cascade_start = detach_dock(layout, &mut undo, source_id)?;
    }

    // 4. Attach. The target's slot is re-derived *after* the detach
    //    (removing a left sibling shifts the target's index). The
    //    resolved placement is a root, split-child, host-edge, or
    //    floating slot — never a tab-member slot (resolution walked
    //    those up).
    let (container, created) = match layout.placement(resolved) {
        Some(DockSlot::Root) => {
            // A new container inherits the root slot; the previous root
            // (still the resolved target) becomes its child.
            let n = create_split_around(layout, resolved, source_id, direction);
            layout.set_root_unchecked(Some(n));
            (n, true)
        }
        Some(DockSlot::SplitChild { parent, index }) => {
            // Reused container when the parent's orientation matches the
            // direction; otherwise a new container wraps the target and
            // inherits the target's slot inside the parent.
            let orientation = layout
                .get(parent)
                .and_then(|dock| dock.orientation())
                .expect("split parent sits in a SplitChild slot");
            if orientation == direction.orientation() {
                record_split(layout, &mut undo, parent);
                let insert_index = if direction.inserts_after() { index + 1 } else { index };
                let target_weight = layout
                    .get(parent)
                    .and_then(|dock| dock.sizes())
                    .map(|sizes| sizes.weights()[index])
                    .expect("split child index is inside the weight list");
                if let Some(dock) = layout.get_mut(parent) {
                    if let DockRole::Split { children, sizes, .. } = &mut dock.role {
                        children.insert(insert_index, source_id);
                        // Sizing baseline: the new child copies the
                        // target's weight (siblings unchanged) so source
                        // and target share the target's former area.
                        sizes.insert_weight(insert_index, target_weight);
                    }
                }
                (parent, false)
            } else {
                let n = create_split_around(layout, resolved, source_id, direction);
                record_split(layout, &mut undo, parent);
                if let Some(dock) = layout.get_mut(parent) {
                    if let DockRole::Split { children, .. } = &mut dock.role {
                        children[index] = n;
                        // The slot's weight stays: `n` inherits the
                        // target's share (PRD §9.2 baseline).
                    }
                }
                (n, true)
            }
        }
        Some(DockSlot::HostEdge { host, edge }) => {
            let n = create_split_around(layout, resolved, source_id, direction);
            record_region(layout, &mut undo, host, edge);
            // The fresh split inherits the region's whole slot: the
            // occupant *and* the slot's sizing weight (PRD §9.2 slot
            // inheritance, mirrored for regions in §9.5).
            let weight = layout
                .get(host)
                .and_then(|dock| dock.host_regions())
                .map(|regions| regions.slot_weight(edge))
                .expect("host-edge slot always names an existing host");
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                    regions.assign(edge, Some(n), weight);
                }
            }
            (n, true)
        }
        // A floating dock has no directional layout surface in v1
        // (floating rearrangement is Phase 8, PRD §8.6): reject.
        Some(DockSlot::Floating { .. }) => {
            return Err(DockingError::NotSplittable { id: resolved })
        }
        // Reachable only for invalid cyclic tab chains (the resolution
        // walk broke out); reported as what it is.
        Some(DockSlot::TabMember { .. }) => {
            return Err(DockingError::CycleDetected { id: resolved })
        }
        // The dock exists (checked above), so a missing placement is an
        // orphan — a state committed layouts cannot hold (PRD DM-07).
        None => return Err(DockingError::OrphanDock { id: resolved }),
    };

    // 5. Collapse the former parent (never the destination: the source
    //    was already attached above).
    if let Some(parent) = cascade_start {
        collapse_cascade(layout, &mut undo, parent)?;
    }

    if created {
        undo.created.push(container);
    }
    Ok(DockingOutcome::InsertedBeside { source: source_id, target, container, inverse: undo })
}

/// Create an unplaced split container holding [`target_dock`, source]
/// ordered by `direction` with uniform sizes, and return its id.
///
/// Uses [`DockLayout::put_split`]: the wrapped target is *placed* (it is
/// being re-parented into the fresh container), which the public
/// `create_split` preconditions would reject (PRD §8.6 wrap case).
fn create_split_around(
    layout: &mut DockLayout,
    target_dock: DockId,
    source: DockId,
    direction: SplitDirection,
) -> DockId {
    let children = if direction.inserts_after() {
        vec![target_dock, source]
    } else {
        vec![source, target_dock]
    };
    let sizes = SplitSizes::uniform(2).expect("a two-child split has a uniform distribution");
    layout.put_split(direction.orientation(), children, sizes)
}

/// `TabJoin` (PRD §8.7). Preconditions: the target exists and is a panel;
/// for an existing source — it exists, is a panel, and the target is not
/// the source or inside its subtree (cycle). Behaviour: when the target
/// already sits in a tab group the source joins it (as the last member,
/// becoming active); otherwise a group is created around both and
/// inherits the target's slot. A source already inside the target's
/// group is a no-op success. The former parent is left valid via the
/// collapse cascade; failure leaves the layout untouched.
fn apply_tab_join(
    layout: &mut DockLayout,
    source: InsertSource,
    target: DockId,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    if !layout.contains(target) {
        return Err(DockingError::DockNotFound { id: target });
    }
    if layout.kind_of(target) != Some(super::dock::DockKind::Panel) {
        return Err(DockingError::IneligibleDock { id: target });
    }
    let is_existing = matches!(source, InsertSource::Existing(_));
    let source_id = match source {
        InsertSource::Existing(id) => {
            if !layout.contains(id) {
                return Err(DockingError::DockNotFound { id });
            }
            if layout.kind_of(id) != Some(super::dock::DockKind::Panel) {
                return Err(DockingError::IneligibleDock { id });
            }
            if layout.is_descendant(target, id) {
                return Err(DockingError::CycleDetected { id });
            }
            id
        }
        InsertSource::NewPanel { content } => {
            // Same undo ownership as `InsertBeside`: a new source panel
            // is removed again on undo.
            let id = layout.create_panel(content);
            undo.created.push(id);
            id
        }
    };

    // The target's *direct* group (the join is into the group the target
    // is a member of, wherever the group itself sits).
    let group = match layout.placement(target) {
        Some(DockSlot::TabMember { parent, .. }) => Some(parent),
        _ => None,
    };

    // Already a member of the target's group: no structural change
    // (tab repositioning goes through ReorderTab, PRD §8.7).
    if let Some(g) = group {
        if let Some(DockRole::TabGroup { members, .. }) =
            layout.get(g).map(|dock| &dock.role)
        {
            if members.contains(&source_id) {
                return Ok(DockingOutcome::TabJoined {
                    source: source_id,
                    target,
                    group: g,
                    inverse: undo,
                });
            }
        }
    }

    // Detach (existing sources only) after the no-op check.
    let mut cascade_start = None;
    if is_existing {
        cascade_start = detach_dock(layout, &mut undo, source_id)?;
    }

    let (group, created) = match group {
        Some(g) => {
            record_tab(layout, &mut undo, g);
            if let Some(dock) = layout.get_mut(g) {
                if let DockRole::TabGroup { members, active, .. } = &mut dock.role {
                    members.push(source_id);
                    // Baseline: the newly joined member becomes active.
                    *active = members.len() - 1;
                }
            }
            (g, false)
        }
        None => {
            // The target's *original* slot must be captured before the
            // wrap installs the group: `placement` derives tab membership
            // by scan, so once the group contains the target the slot
            // reads as the group's first member, not the slot to inherit.
            let target_slot = layout
                .placement(target)
                .expect("the target was detached from nothing (it stays placed)");
            let g = layout.put_tab_group(vec![target, source_id], 1);
            match target_slot {
                DockSlot::Root => {
                    layout.set_root_unchecked(Some(g));
                }
                DockSlot::SplitChild { parent, index } => {
                    record_split(layout, &mut undo, parent);
                    if let Some(dock) = layout.get_mut(parent) {
                        if let DockRole::Split { children, .. } = &mut dock.role {
                            children[index] = g;
                        }
                    }
                }
                DockSlot::HostEdge { host, edge } => {
                    record_region(layout, &mut undo, host, edge);
                    // The group inherits the region's whole slot
                    // (occupant *and* sizing weight — the §9.5 slot
                    // inheritance).
                    let weight = layout
                        .get(host)
                        .and_then(|dock| dock.host_regions())
                        .map(|regions| regions.slot_weight(edge))
                        .expect("host-edge slot names an existing host");
                    if let Some(dock) = layout.get_mut(host) {
                        if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                            regions.assign(edge, Some(g), weight);
                        }
                    }
                }
                DockSlot::Floating { host, index } => {
                    record_floating(layout, &mut undo, host);
                    if let Some(dock) = layout.get_mut(host) {
                        if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                            // The group replaces the target in the collection;
                            // the entry (and its rect — the slot's state carried
                            // forward, the §9.6 slot inheritance) stays put.
                            floating[index].dock = g;
                        }
                    }
                }
                // Impossible: `target_slot` was captured while the target
                // still occupied one of the slots above (a tab-member
                // slot resolves to the `Some(g)` branch).
                DockSlot::TabMember { .. } => {
                    unreachable!("tab-member targets always resolve to a group")
                }
            }
            (g, true)
        }
    };

    if let Some(parent) = cascade_start {
        collapse_cascade(layout, &mut undo, parent)?;
    }

    if created {
        undo.created.push(group);
    }
    Ok(DockingOutcome::TabJoined { source: source_id, target, group, inverse: undo })
}

/// `ReorderTab` — Layers `reorder_sibling` convention (insertion index
/// validated against the pre-removal member count; `from == to` is a
/// no-op success).
fn apply_reorder_tab(
    layout: &mut DockLayout,
    group: DockId,
    from: usize,
    to: usize,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    if !layout.contains(group) {
        return Err(DockingError::DockNotFound { id: group });
    }
    let Some(dock) = layout.get(group) else {
        return Err(DockingError::DockNotFound { id: group });
    };
    if dock.kind() != super::dock::DockKind::TabGroup {
        return Err(DockingError::NotATabGroup { id: group });
    }
    let Some(DockRole::TabGroup { members, .. }) = layout.get(group).map(|d| &d.role) else {
        return Err(DockingError::NotATabGroup { id: group });
    };
    if from >= members.len() || to >= members.len() {
        return Err(DockingError::TabIndexOutOfRange {
            index: if from >= members.len() { from } else { to },
            member_count: members.len(),
        });
    }
    if from == to {
        return Ok(DockingOutcome::TabReordered { group, from, to, inverse: undo });
    }

    record_tab(layout, &mut undo, group);
    if let Some(dock) = layout.get_mut(group) {
        if let DockRole::TabGroup { members, active, .. } = &mut dock.role {
            let member = members.remove(from);
            members.insert(to, member);
            if from == *active {
                // The moved member carries its selection.
                *active = to;
            } else {
                if from < *active {
                    *active -= 1;
                }
                if to <= *active {
                    *active += 1;
                }
            }
        }
    }
    Ok(DockingOutcome::TabReordered { group, from, to, inverse: undo })
}

/// `SetActiveTab` (PRD §8.7 active-selection validity).
fn apply_set_active_tab(
    layout: &mut DockLayout,
    group: DockId,
    index: usize,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    if !layout.contains(group) {
        return Err(DockingError::DockNotFound { id: group });
    }
    let Some(dock) = layout.get(group) else {
        return Err(DockingError::DockNotFound { id: group });
    };
    if dock.kind() != super::dock::DockKind::TabGroup {
        return Err(DockingError::NotATabGroup { id: group });
    }
    let Some(DockRole::TabGroup { members, .. }) = layout.get(group).map(|d| &d.role) else {
        return Err(DockingError::NotATabGroup { id: group });
    };
    if index >= members.len() {
        return Err(DockingError::ActiveIndexOutOfRange {
            index,
            member_count: members.len(),
        });
    }
    record_tab(layout, &mut undo, group);
    if let Some(dock) = layout.get_mut(group) {
        if let DockRole::TabGroup { active, .. } = &mut dock.role {
            *active = index;
        }
    }
    Ok(DockingOutcome::ActiveTabSelected { group, active: index, inverse: undo })
}

/// `Remove` (PRD §8.11 semantics (b)): detach the dock, delete its whole
/// descendant subtree (BFS order: the dock first, then descendants), and
/// cascade-normalize the vacated parent chain. Content lifetime stays
/// with the host application (PRD §8.11).
fn apply_remove(
    layout: &mut DockLayout,
    dock: DockId,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    if !layout.contains(dock) {
        return Err(DockingError::DockNotFound { id: dock });
    }

    // Collect the subtree in BFS order (dock first, then descendants)
    // *before* any mutation.
    let mut subtree = vec![dock];
    subtree.extend(layout.descendants(dock));
    let removed_docks: Vec<Dock> = subtree
        .iter()
        .filter_map(|id| layout.get(*id).cloned())
        .collect();

    // Detach (records the former parent), then delete the payloads.
    let cascade_start = detach_dock(layout, &mut undo, dock)?;
    for id in &subtree {
        if let Some(payload) = layout.take_dock(*id) {
            undo.removed.push(payload);
        }
    }
    if let Some(parent) = cascade_start {
        collapse_cascade(layout, &mut undo, parent)?;
    }

    Ok(DockingOutcome::Removed { docks: removed_docks, inverse: undo })
}

/// `ResizeSiblings` (PRD §9.3–§9.4). Preconditions, in check order: the
/// container exists and is a split; the pair start `first` has an
/// adjacent sibling; the resulting pair weights satisfy the carried
/// minimum bounds and the sizing invariant. Behaviour: the pair is
/// rewritten (first := `requested`; second := pair total − `requested`),
/// every other child's weight verbatim — a non-structural change, so
/// the normalization cascade is skipped. Failure leaves the layout
/// untouched; the constraint policy is rejection (never clamping, PRD
/// §9.3).
fn apply_resize_siblings(
    layout: &mut DockLayout,
    container: DockId,
    first: usize,
    requested: f64,
    first_min: Option<f64>,
    second_min: Option<f64>,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    // 1. Container existence and category.
    let Some(dock) = layout.get(container) else {
        return Err(DockingError::DockNotFound { id: container });
    };
    if dock.kind() != super::dock::DockKind::Split {
        return Err(DockingError::NotASplit { id: container });
    }
    let DockRole::Split { children, sizes, .. } = &dock.role else {
        unreachable!("split category was just checked")
    };
    // 2. The pair must exist in the child (weight) order. Guarded
    //    comparison keeps `usize::MAX` arguments from overflowing.
    let child_count = children.len();
    if child_count < 2 || first >= child_count - 1 {
        return Err(DockingError::SplitChildIndexOutOfRange { index: first, child_count });
    }
    let previous_pair = [sizes.weights()[first], sizes.weights()[first + 1]];
    let pair_total = previous_pair[0] + previous_pair[1];

    // 3. Constraint policy (PRD §9.3): the pair's regions are checked in
    //    ascending child order — deterministic reporting. Each region
    //    must land on a finite, strictly positive weight (the sizing
    //    invariant) that satisfies its minimum inclusively; a NaN or
    //    infinite minimum is unsatisfiable like any other bound. A
    //    stored pair that cannot produce a finite complement (imported
    //    garbage) rejects the request the same deterministic way.
    let resulting = [requested, pair_total - requested];
    for (offset, &weight) in resulting.iter().enumerate() {
        let min = if offset == 0 { first_min } else { second_min };
        let violates = !weight.is_finite() || weight <= 0.0 || min.is_some_and(|m| !(weight >= m));
        if violates {
            return Err(DockingError::SizeBoundViolation {
                split: container,
                index: first + offset,
            });
        }
    }

    // 4. Record the former state, then rewrite exactly the pair. Stored
    //    weights outside the pair could be invalid imported garbage;
    //    named via the steady-state authority like `prune_weight` (the
    //    shifted-removal precedent) so the rejection is deterministic
    //    and the layout untouched.
    record_split(layout, &mut undo, container);
    let mut weights = sizes.weights().to_vec();
    weights[first] = requested;
    weights[first + 1] = resulting[1];
    let Some(next) = SplitSizes::from_weights(weights) else {
        let offender = sizes
            .weights()
            .iter()
            .enumerate()
            .find(|(i, w)| *i != first && *i != first + 1 && (!w.is_finite() || **w <= 0.0))
            .map(|(i, _)| i)
            .unwrap_or(first);
        return Err(DockingError::InvalidSizeState { split: container, index: offender });
    };
    if let Some(dock) = layout.get_mut(container) {
        if let DockRole::Split { sizes, .. } = &mut dock.role {
            *sizes = next;
        }
    }
    Ok(DockingOutcome::SiblingsResized { container, first, previous: previous_pair, inverse: undo })
}

/// `SetDockCollapsed` (PRD §9.7). Preconditions: the dock exists and is
/// a panel (the v1 eligible category). Behaviour: the panel's collapse
/// preference flips; no placement, membership, or sizing data changes,
/// so the dock stays placed, traversable, and restorable. Setting the
/// current preference is a no-op success whose inverse is the identity.
fn apply_set_dock_collapsed(
    layout: &mut DockLayout,
    dock: DockId,
    collapsed: bool,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    let Some(existing) = layout.get(dock) else {
        return Err(DockingError::DockNotFound { id: dock });
    };
    if existing.kind() != super::dock::DockKind::Panel {
        return Err(DockingError::IneligibleDock { id: dock });
    }
    let Some(DockRole::Panel { collapsed: previous, .. }) = layout.get(dock).map(|d| &d.role)
    else {
        return Err(DockingError::IneligibleDock { id: dock });
    };
    let previous = *previous;
    if previous == collapsed {
        // No-op success (the ReorderTab `from == to` precedent): the
        // inverse stays the identity, so replay restores nothing.
        return Ok(DockingOutcome::DockCollapsedSet {
            dock,
            collapsed,
            previous,
            inverse: undo,
        });
    }
    undo.containers.push(FormerCollection::Panel { id: dock, collapsed: previous });
    if let Some(existing) = layout.get_mut(dock) {
        if let DockRole::Panel { collapsed: preference, .. } = &mut existing.role {
            *preference = collapsed;
        }
    }
    Ok(DockingOutcome::DockCollapsedSet { dock, collapsed, previous, inverse: undo })
}

// --- Embedding and host lifecycle (Phase 7) ---

/// Promote `dock` to embed-host status (PRD §8.9): the panel's role
/// becomes a fresh-empty host with `persistent` as given — identity,
/// content, and placement untouched. `Embed`'s auto-promotion passes
/// `false` (a temporary promotion created solely to support embedding —
/// demoted back once its last embedded/floating dock leaves); the
/// explicit `PromoteHost` operation passes `true` (an intentionally
/// persistent host). Records the pre-promotion payload in `undo`. The
/// caller checked the eligibility (the dock is a panel).
fn promote_to_host(layout: &mut DockLayout, undo: &mut UndoData, dock: DockId, persistent: bool) {
    record_promotion(layout, undo, dock);
    if let Some(existing) = layout.get_mut(dock) {
        if let DockRole::Panel { content, .. } = &existing.role {
            let content = content.clone();
            existing.role = DockRole::EmbedHost {
                content,
                regions: HostRegions::new(),
                floating: Vec::new(),
                persistent,
            };
        }
    }
}

/// `Embed` (PRD §8.8–§8.9). Preconditions, in check order: the host
/// exists and is a work-area dock (embed host, or a panel to
/// auto-promote; splits/tab groups reject `IneligibleDock`); for an
/// existing source — it exists, and the host is not the source or
/// inside the source's subtree (`CycleDetected`: the attach would make
/// the source its own descendant; a source *inside* the host's subtree
/// re-parents into another slot of its own ancestor and is allowed);
/// the requested edge region is unoccupied (`RegionOccupied` — the
/// §8.8 baseline: never silently replaced). Behaviour: the source
/// detaches (cascade), the host promotes when it was a panel, and the
/// source takes the region (sizing baseline `1.0`) or appends to the
/// floating collection. Failure leaves the layout untouched.
fn apply_embed(
    layout: &mut DockLayout,
    source: InsertSource,
    host: DockId,
    slot: EmbedSlot,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    // 1. Host existence and hosting eligibility (a named work area).
    let Some(host_dock) = layout.get(host) else {
        return Err(DockingError::DockNotFound { id: host });
    };
    match host_dock.kind() {
        DockKind::EmbedHost | DockKind::Panel => {}
        // Splits and tab groups hold no work area: they can neither
        // host nor promote (PRD §8.9 promotion preserves the work area).
        DockKind::Split | DockKind::TabGroup => {
            return Err(DockingError::IneligibleDock { id: host })
        }
    }

    // 2. Source existence and the cycle check. Embedding places the
    //    source into a pre-existing slot of the host, so the only
    //    cycle hazard is the **host sitting inside the source's
    //    subtree** (including the source being the host): the attach
    //    would make the source its own descendant. A source *inside*
    //    the host's subtree is fine — it re-parents into another slot
    //    of its own ancestor (e.g. a region occupant undocked to
    //    floating).
    let is_existing = matches!(source, InsertSource::Existing(_));
    let source_id = match source {
        InsertSource::Existing(id) => {
            if !layout.contains(id) {
                return Err(DockingError::DockNotFound { id });
            }
            if layout.is_descendant(host, id) {
                return Err(DockingError::CycleDetected { id: host });
            }
            id
        }
        InsertSource::NewPanel { content } => {
            let id = layout.create_panel(content);
            undo.created.push(id);
            id
        }
    };

    // 3. Occupancy and initial-state checks before any mutation (PRD
    //    §8.8, §9.6). An occupied region rejects the embed; a floating
    //    append accepts every appender but must carry a valid initial
    //    rect (the PRD §9.6 host-bounds rule — the drop interaction's
    //    state, rejections name the would-be entry index).
    let initial_rect = match slot {
        EmbedSlot::Region(edge) => {
            let occupied = layout
                .get(host)
                .and_then(|dock| dock.host_regions())
                .is_some_and(|regions| regions.on(edge).is_some());
            if occupied {
                return Err(DockingError::RegionOccupied { host, edge });
            }
            None
        }
        EmbedSlot::Floating { rect } => {
            let proposed = rect.unwrap_or_default(); // baseline when None
            if !proposed.is_valid() {
                let index = layout
                    .get(host)
                    .and_then(|dock| dock.floating())
                    .map(|floating| floating.len())
                    .unwrap_or(0);
                return Err(DockingError::FloatingBoundViolation { host, index });
            }
            Some(proposed)
        }
    };

    // 4. Detach the source (records its former parent). A new panel
    //    has nothing to detach.
    let mut cascade_start = None;
    if is_existing {
        cascade_start = detach_dock(layout, &mut undo, source_id)?;
    }

    // 5. Promotion when the host was a panel (PRD §8.9 "when a valid
    //    embedding operation requires it"): identity and content are
    //    preserved; the fresh-empty temporary host then takes the
    //    embed. (The occupancy check read the pre-promotion panel — a
    //    panel's regions are empty, so it cannot have rejected.)
    let promoted = layout.kind_of(host) == Some(DockKind::Panel);
    if promoted {
        // Auto-promotion is temporary: the panel was promoted solely to
        // support this embedding, so normalization demotes it back once
        // nothing is embedded in it (PRD §8.9 baseline).
        promote_to_host(layout, &mut undo, host, false);
    }

    // 6. Attach. Records run in mutation order; undo replay (newest
    //    first) applies the role-restoring promotion record last.
    match slot {
        EmbedSlot::Region(edge) => {
            record_region(layout, &mut undo, host, edge);
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                    // Sizing baseline: the embedding weight 1.0 —
                    // uniform with the implicit core weight (§9.5).
                    regions.assign(edge, Some(source_id), REGION_WEIGHT_BASELINE);
                }
            }
        }
        EmbedSlot::Floating { .. } => {
            record_floating(layout, &mut undo, host);
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                    // Appended last (the collection order is placement
                    // order; visual stacking is frontend-derived,
                    // PRD §11.3). Sizing: the entry takes its initial
                    // rect — the `EmbedSlot`'s rect, or the floating
                    // baseline when none was supplied (PRD §9.6).
                    let rect = initial_rect.expect("the floating arm always proposes a rect");
                    floating.push(FloatingEntry::new(source_id, rect));
                }
            }
        }
    }

    // 7. Cascade-normalize the vacated parent chain. The pass also
    //    demotes a temporary host the detach emptied (PRD §8.9).
    if let Some(parent) = cascade_start {
        collapse_cascade(layout, &mut undo, parent)?;
    }

    Ok(DockingOutcome::Embedded { source: source_id, host, slot, promoted, inverse: undo })
}

/// `PromoteHost` (PRD §8.9). Preconditions: the dock exists and is a
/// work-area dock (panel; splits/tab groups reject `IneligibleDock`).
/// Behaviour: the panel becomes a fresh-empty **persistent** host with
/// identity, content, and placement untouched — an explicit promotion is
/// an intentionally persistent host (the PRD §8.9 distinction), so it
/// remains a host even when its regions and floating collection are
/// empty; only `Embed`'s auto-promotion is temporary. An already-host
/// dock is a no-op success (identity inverse, the `SetDockCollapsed`
/// matching-value precedent).
fn apply_promote_host(layout: &mut DockLayout, dock: DockId) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    let Some(existing) = layout.get(dock) else {
        return Err(DockingError::DockNotFound { id: dock });
    };
    match existing.kind() {
        DockKind::EmbedHost => {
            // Already a host (persistent or temporary): no-op success.
            return Ok(DockingOutcome::HostPromoted { dock, promoted: false, inverse: undo });
        }
        DockKind::Panel => {}
        DockKind::Split | DockKind::TabGroup => {
            return Err(DockingError::IneligibleDock { id: dock })
        }
    }

    promote_to_host(layout, &mut undo, dock, true);
    Ok(DockingOutcome::HostPromoted { dock, promoted: true, inverse: undo })
}

/// `ResizeRegion` (PRD §9.5). Preconditions, in check order: the host
/// exists and is an embed host; the region is occupied; the requested
/// weight satisfies the sizing invariant and the carried minimum
/// (rejection policy — never clamped, PRD §9.3). Behaviour: exactly the
/// named region's stored weight is rewritten — the occupant and every
/// other stored weight of the host stay verbatim, so no other placement
/// is invalidated (the §9.5 coherence rule). Failure leaves the layout
/// untouched.
fn apply_resize_region(
    layout: &mut DockLayout,
    host: DockId,
    edge: EdgeRegion,
    requested: f64,
    min: Option<f64>,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    // 1. Host existence and category.
    let Some(dock) = layout.get(host) else {
        return Err(DockingError::DockNotFound { id: host });
    };
    if dock.kind() != DockKind::EmbedHost {
        return Err(DockingError::NotAnEmbedHost { id: host });
    }

    // 2. Occupancy: only an occupied region holds a resizing boundary
    //    (and its preference — weights live with the region lifecycle).
    let Some(regions) = dock.host_regions() else {
        unreachable!("embed-host category was just checked")
    };
    let Some(previous) = regions.weight_on(edge) else {
        return Err(DockingError::RegionUnoccupied { host, edge });
    };

    // 3. Constraint policy (§9.3, as `ResizeSiblings`): the sizing
    //    invariant is always the implicit bound; a carried minimum is
    //    honored inclusively.
    let violates = !requested.is_finite() || requested <= 0.0 || min.is_some_and(|m| !(requested >= m));
    if violates {
        return Err(DockingError::RegionBoundViolation { host, edge });
    }

    // 4. Record the former region state, then rewrite exactly the one
    //    region weight. The write covers the whole field, so there is
    //    no rebuilding step that could trip over other stored garbage
    //    (unlike the split vector): imported garbage is the Phase-3
    //    gate's steady-state finding.
    record_region(layout, &mut undo, host, edge);
    if let Some(dock) = layout.get_mut(host) {
        if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
            regions.set_weight(edge, requested);
        }
    }
    Ok(DockingOutcome::RegionResized { host, edge, previous, inverse: undo })
}

/// `SetFloatingRect` (PRD §9.6, Phase 8). Preconditions, in check order:
/// the host exists and is an embed host; the entry index exists in the
/// host's floating collection; the requested rect satisfies the §9.6
/// representation invariant, the host-bounds rule, and the carried
/// minimums (rejection policy — never clamped, PRD §9.3). Behaviour:
/// exactly the named entry's rect is rewritten — the floated dock, its
/// placement, and every other entry of the collection stay verbatim (a
/// non-structural change, so no normalization cascade runs). Failure
/// leaves the layout untouched.
fn apply_set_floating_rect(
    layout: &mut DockLayout,
    host: DockId,
    index: usize,
    rect: FloatingRect,
    min_w: Option<f64>,
    min_h: Option<f64>,
) -> Result<DockingOutcome, DockingError> {
    let mut undo = UndoData::identity(layout);

    // 1. Host existence and category.
    let Some(dock) = layout.get(host) else {
        return Err(DockingError::DockNotFound { id: host });
    };
    if dock.kind() != DockKind::EmbedHost {
        return Err(DockingError::NotAnEmbedHost { id: host });
    }

    // 2. The entry must exist (the floating-side index rule).
    let entries = dock.floating().expect("embed-host category was just checked");
    let entry_count = entries.len();
    let Some(entry) = entries.get(index) else {
        return Err(DockingError::FloatingIndexOutOfRange { index, entry_count });
    };
    let previous = entry.rect();

    // 3. Constraint policy (§9.3): the rect invariant (finite parts,
    //    non-empty, within the host's logical bounds — PRD §9.6 SHOULD,
    //    endpoints inclusive) is always the implicit bound; the carried
    //    minimums are honored inclusively. A NaN minimum is
    //    unsatisfiable like any other bound; the rejection names the
    //    host and entry, deterministically, and never clamps.
    let violates = !rect.is_valid()
        || min_w.is_some_and(|m| !(rect.w() >= m))
        || min_h.is_some_and(|m| !(rect.h() >= m));
    if violates {
        return Err(DockingError::FloatingBoundViolation { host, index });
    }

    // 4. Record the former collection state, then rewrite exactly the
    //    one entry's rect (the `ResizeRegion` write shape — the whole
    //    rect field, no rebuild step that could trip over other stored
    //    garbage; imported garbage is the Phase-3 gate's steady-state
    //    finding).
    record_floating(layout, &mut undo, host);
    if let Some(dock) = layout.get_mut(host) {
        if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
            floating[index].set_rect(rect);
        }
    }
    Ok(DockingOutcome::FloatingRectSet { host, index, previous, inverse: undo })
}

#[cfg(test)]
mod tests {
    //! In-crate inverse-replay round-trips: the outcomes must restore
    //! the pre-operation layout exactly (engine undo contract, Ph.1
    //! R-A1). Undo uses `pub(crate)` seams (`replay_into`) the same way
    //! the Phase-9 `ExecutedDockingOperation` wrapper will. Placement
    //! surgery via `pub(crate)` mirrors imported (restored) states that
    //! host edge regions / floating collections occupied — the public
    //! creators are the `Embed` routes (Phases 7–8); the fixtures keep
    //! to the minimal direct route for below-minimum subtrees.

    use super::*;

    fn panel(id: u64) -> DockId {
        DockId::from_raw(id)
    }

    /// Structural state comparison for undo round-trips: docks and root
    /// must reconstruct exactly. The id counter is deliberately **not**
    /// compared — it is monotone by design (never reset, not even by
    /// `clear`; undo of a *created* dock leaves the counter incremented,
    /// the same convention as `Canvas::insert_object` undo).
    fn structural_equal(a: &DockLayout, b: &DockLayout) -> bool {
        let snapshot = |layout: &DockLayout| {
            let mut docks: Vec<Dock> = layout.iter().cloned().collect();
            docks.sort_by_key(|dock| dock.id);
            (layout.root(), docks)
        };
        snapshot(a) == snapshot(b)
    }

    #[test]
    fn inserted_beside_round_trips_through_undo() {
        let layout_before = {
            let mut layout = DockLayout::new();
            let a = layout.create_panel(Some("a".into()));
            let b = layout.create_panel(Some("b".into()));
            layout.set_root(None).unwrap();
            let row = layout
                .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
                .unwrap();
            layout.set_root(Some(row)).unwrap();
            assert!(layout.is_valid());
            layout
        };

        // Move an existing sibling: detach + insert into the same
        // container (double-mutation of one collection).
        let mut layout = layout_before.clone();
        let outcome = DockingOperation::InsertBeside {
            source: InsertSource::Existing(panel(1)),
            target: panel(0),
            direction: SplitDirection::Left,
        }
        .apply(&mut layout)
        .unwrap();
        assert!(layout.is_valid());
        outcome.inverse().clone().replay_into(&mut layout);
        assert_eq!(layout, layout_before);
    }

    #[test]
    fn removed_subtree_with_cascade_round_trips_through_undo() {
        let layout_before = {
            let mut layout = DockLayout::new();
            let a = layout.create_panel(Some("a".into()));
            let b = layout.create_panel(Some("b".into()));
            layout.set_root(None).unwrap();
            let inner = layout
                .create_split(
                    SplitOrientation::Column,
                    vec![a, b],
                    SplitSizes::uniform(2).unwrap(),
                )
                .unwrap();
            let c = layout.create_panel(Some("c".into()));
            let outer = layout
                .create_split(
                    SplitOrientation::Row,
                    vec![inner, c],
                    SplitSizes::uniform(2).unwrap(),
                )
                .unwrap();
            layout.set_root(Some(outer)).unwrap();
            assert!(layout.is_valid());
            layout
        };

        // Removing `a` collapses `inner` (one child left) and the
        // rewrite of `outer` must be replayed backwards, too.
        let mut layout = layout_before.clone();
        let outcome = DockingOperation::Remove { dock: panel(0) }.apply(&mut layout).unwrap();
        assert!(layout.is_valid());
        assert_eq!(outcome.inverse().removed.len(), 2); // a + collapsed inner
        outcome.inverse().clone().replay_into(&mut layout);
        assert_eq!(layout, layout_before);
    }

    #[test]
    fn created_tab_group_with_reused_parents_round_trips_through_undo() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.set_root(None).unwrap();
        let row = layout
            .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
            .unwrap();
        layout.set_root(Some(row)).unwrap();
        let before = layout.clone();

        let outcome = DockingOperation::TabJoin {
            source: InsertSource::Existing(panel(1)),
            target: panel(0),
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::TabJoined { group, inverse, .. } = &outcome else {
            panic!("expected TabJoined");
        };
        assert_ne!(*group, row);
        assert!(layout.is_valid());
        inverse.clone().replay_into(&mut layout);
        assert!(structural_equal(&layout, &before));
    }

    #[test]
    fn insert_beside_region_occupant_round_trips_through_undo() {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(Some("host".into()));
        let occupant = layout.create_panel(None);
        {
            // Import-style surgery: an occupied edge region on the host
            // (the embedding operation is the public route; this keeps
            // the undo-replay fixture minimal).
            let dock = layout.get_mut(host).expect("host exists");
            if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                regions.assign(EdgeRegion::Top, Some(occupant), 1.0);
            }
        }
        assert!(layout.is_valid());
        let before = layout.clone();

        // Dropping a new panel on top of the occupant splits the region:
        // a fresh split inherits the region slot.
        let outcome = DockingOperation::InsertBeside {
            source: InsertSource::NewPanel { content: Some("n".into()) },
            target: occupant,
            direction: SplitDirection::Top,
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::InsertedBeside { container, inverse, .. } = outcome else {
            panic!("expected InsertedBeside");
        };
        assert!(layout.is_valid());
        assert!(
            layout.get(host).unwrap().host_regions().unwrap().on(EdgeRegion::Top)
                == Some(container)
        );
        // The region record restores the pre-op occupant exactly.
        inverse.replay_into(&mut layout);
        assert!(structural_equal(&layout, &before));
    }

    #[test]
    fn insert_beside_floating_target_is_rejected() {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(None);
        let floater = layout.create_panel(None);
        {
            let dock = layout.get_mut(host).expect("host exists");
            if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                // Import-style surgery: the floater enters at the
                // baseline rect (the public route is `Embed` — the
                // fixture keeps the minimal install).
                floating.push(FloatingEntry::new(floater, FloatingRect::default()));
            }
        }
        assert!(layout.is_valid());
        let before = layout.clone();

        // Floating docks have no directional layout surface in v1.
        let err = DockingOperation::InsertBeside {
            source: InsertSource::NewPanel { content: None },
            target: floater,
            direction: SplitDirection::Left,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::NotSplittable { id: floater });
        // Op-01/OP-02: the failed operation leaves the layout as it was.
        assert_eq!(layout, before);
    }

    #[test]
    fn resize_pair_round_trips_through_undo() {
        let layout_before = {
            let mut layout = DockLayout::new();
            let a = layout.create_panel(Some("a".into()));
            let b = layout.create_panel(Some("b".into()));
            layout.set_root(None).unwrap();
            let row = layout
                .create_split(
                    SplitOrientation::Row,
                    vec![a, b],
                    SplitSizes::from_weights(vec![0.7, 0.3]).unwrap(),
                )
                .unwrap();
            layout.set_root(Some(row)).unwrap();
            assert!(layout.is_valid());
            layout
        };

        let mut layout = layout_before.clone();
        let outcome = DockingOperation::ResizeSiblings {
            container: DockId::from_raw(2),
            first: 0,
            requested: 0.5,
            first_min: None,
            second_min: None,
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::SiblingsResized { previous, inverse, .. } = &outcome else {
            panic!("expected SiblingsResized");
        };
        assert_eq!(*previous, [0.7, 0.3]);
        assert!(layout.is_valid());
        inverse.clone().replay_into(&mut layout);
        assert_eq!(layout, layout_before);
    }

    #[test]
    fn collapsed_preference_round_trips_through_undo() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(Some("a".into()));
        let b = layout.create_panel(Some("b".into()));
        layout.set_root(None).unwrap();
        let row = layout
            .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
            .unwrap();
        layout.set_root(Some(row)).unwrap();
        let before = layout.clone();

        let outcome = DockingOperation::SetDockCollapsed { dock: a, collapsed: true }
            .apply(&mut layout)
            .unwrap();
        let DockingOutcome::DockCollapsedSet { dock, collapsed, previous, inverse } = &outcome
        else {
            panic!("expected DockCollapsedSet");
        };
        assert_eq!(*dock, a);
        assert!(*collapsed && !*previous);
        // No structural or sizing data changed: only the flag flipped.
        assert_eq!(
            layout.get(row).unwrap().sizes().map(|s| s.weights().to_vec()),
            Some(vec![1.0, 1.0])
        );
        assert!(layout.is_valid());
        inverse.clone().replay_into(&mut layout);
        assert_eq!(layout, before);
    }

    #[test]
    fn resize_with_imported_garbage_sizing_is_deterministic() {
        // Imported (restored) layouts can carry invalid stored weights;
        // both garbage positions must reject deterministically with the
        // layout untouched, exactly the way `prune_weight` behaves.
        let build = || {
            let mut layout = DockLayout::new();
            let a = layout.create_panel(Some("a".into()));
            let b = layout.create_panel(Some("b".into()));
            let c = layout.create_panel(Some("c".into()));
            layout.set_root(None).unwrap();
            let row = layout
                .create_split(
                    SplitOrientation::Row,
                    vec![a, b, c],
                    SplitSizes::uniform(3).unwrap(),
                )
                .unwrap();
            layout.set_root(Some(row)).unwrap();
            layout
        };

        // Garbage outside the pair (index 2): named by the steady-state
        // authority (`InvalidSizeState`), the pair itself stays valid.
        let mut layout = build();
        {
            let dock = layout.get_mut(DockId::from_raw(3)).expect("row exists");
            if let DockRole::Split { sizes, .. } = &mut dock.role {
                *sizes = SplitSizes::from_weights_unchecked(vec![1.0, 2.0, f64::NAN]);
            }
        }
        let before_debug = format!("{layout:?}"); // NaN: compare via Debug
        let err = DockingOperation::ResizeSiblings {
            container: DockId::from_raw(3),
            first: 0,
            requested: 0.5,
            first_min: None,
            second_min: None,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::InvalidSizeState { split: DockId::from_raw(3), index: 2 });
        assert_eq!(format!("{layout:?}"), before_debug);
        let again = DockingOperation::ResizeSiblings {
            container: DockId::from_raw(3),
            first: 0,
            requested: 0.5,
            first_min: None,
            second_min: None,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(again, err);
    }

    // --- Embedding and host lifecycle (Phase 7) ---

    /// A populated root: a persistent embed host `host` (id 0) plus two
    /// plain panels `a` (id 1) and `b` (id 2) in a row split `row` (id
    /// 3) with uniform weights — the shell the lifecycle tests detach
    /// members into a region / the floating collection from.
    fn host_a_b_row() -> (DockLayout, DockId, DockId, DockId, DockId) {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(Some("host".into()));
        let a = layout.create_panel(Some("a".into()));
        let b = layout.create_panel(Some("b".into()));
        layout.set_root(None).unwrap();
        let row = layout.put_split(
            SplitOrientation::Row,
            vec![host, a, b],
            SplitSizes::uniform(3).unwrap(),
        );
        layout.set_root(None).unwrap(); // clear the stale designation
        layout.set_root(Some(row)).unwrap();
        assert!(layout.is_valid());
        (layout, host, a, b, row)
    }

    #[test]
    fn embed_occupied_region_is_rejected_and_leaves_the_layout_untouched() {
        let (mut layout, host, a, _b, _row) = host_a_b_row();
        let DockingOutcome::Embedded { source, .. } = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }
        .apply(&mut layout)
        .unwrap()
        else {
            panic!("expected Embedded");
        };
        assert!(layout.is_valid());
        let between = layout.clone();

        // The occupied region rejects the second embed (PRD §8.8): the
        // existing occupant is never silently discarded.
        let err = DockingOperation::Embed {
            source: InsertSource::NewPanel { content: Some("n".into()) },
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::RegionOccupied { host, edge: EdgeRegion::Top });
        // OP-01/OP-02: the failed operation leaves the layout as it was.
        assert_eq!(layout, between);
        assert_eq!(
            layout.get(host).unwrap().host_regions().unwrap().on(EdgeRegion::Top),
            Some(source)
        );
    }

    #[test]
    fn embed_into_a_split_or_tab_group_is_ineligible() {
        let (mut layout, _host, a, _b, row) = host_a_b_row();
        let before = layout.clone();

        // Splits and tab groups hold no work area: they can neither
        // host nor auto-promote (PRD §8.9).
        let err = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host: row,
            slot: EmbedSlot::Floating { rect: None },
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::IneligibleDock { id: row });
        assert_eq!(layout, before);
        assert!(layout.is_valid());
    }

    #[test]
    fn embed_host_inside_the_source_subtree_is_a_cycle() {
        let mut layout = DockLayout::new();
        let c = layout.create_panel(Some("c".into())); // auto-rooted
        let x = layout.create_panel(Some("x".into()));
        layout.set_root(None).unwrap();
        let inner =
            layout.put_split(SplitOrientation::Row, vec![c, x], SplitSizes::uniform(2).unwrap());
        layout.set_root(Some(inner)).unwrap();
        assert!(layout.is_valid());
        let before = layout.clone();

        // `inner` contains `c`: embedding the split into its own child
        // would make the source its own descendant (the only cycle
        // hazard in `Embed` — the host inside the source).
        let err = DockingOperation::Embed {
            source: InsertSource::Existing(inner),
            host: c,
            slot: EmbedSlot::Region(EdgeRegion::Left),
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::CycleDetected { id: c });
        assert_eq!(layout, before);
    }

    #[test]
    fn embed_source_inside_the_host_subtree_reparents_to_another_slot() {
        // An occupant "undocked" to the floating collection: the source
        // moves into another slot of its own ancestor — the allowed
        // direction (PRD §8.9).
        let (mut layout, host, a, _b, _row) = host_a_b_row();
        let first = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::Embedded { promoted, .. } = &first else {
            panic!("expected Embedded");
        };
        // The host was created persistent — the embed did not promote.
        assert!(!*promoted);
        assert_eq!(layout.get(host).unwrap().persistent(), Some(true));
        let between = layout.clone();

        let anchor = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host,
            slot: EmbedSlot::Floating { rect: None },
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::Embedded { source, slot, .. } = &anchor else {
            panic!("expected Embedded");
        };
        assert_eq!(source, &a);
        assert_eq!(slot, &EmbedSlot::Floating { rect: None });
        assert!(layout.is_valid());
        // The source vacated its region for the floating list.
        assert_eq!(
            layout
                .get(host)
                .unwrap()
                .host_regions()
                .unwrap()
                .on(EdgeRegion::Top),
            None
        );
        // The entry entered at the floating baseline (no rect carried).
        let floating_entries = layout.get(host).unwrap().floating().unwrap();
        assert_eq!(floating_entries.len(), 1);
        assert_eq!(floating_entries[0].dock, a);
        assert_eq!(floating_entries[0].rect(), FloatingRect::default());
        assert_eq!(layout.placement(a), Some(DockSlot::Floating { host, index: 0 }));
        anchor.inverse().clone().replay_into(&mut layout);
        assert_eq!(layout, between);
    }

    #[test]
    fn embed_undo_round_trips_region_and_floating_detaches() {
        let (mut layout, host, a, b, row) = host_a_b_row();
        let before = layout.clone();
        assert_eq!(layout.get(host).unwrap().persistent(), Some(true));

        let o1 = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }
        .apply(&mut layout)
        .unwrap();
        assert!(layout.is_valid());
        let between = layout.clone();

        // Embedding `b` floating detaches it from the row; the row
        // falls below minimum and collapses — its survivor `host`
        // inherits the Root slot (the cascade inside `Embed`).
        let o2 = DockingOperation::Embed {
            source: InsertSource::Existing(b),
            host,
            slot: EmbedSlot::Floating { rect: Some(FloatingRect::new(0.1, 0.2, 0.6, 0.7).unwrap()) },
        }
        .apply(&mut layout)
        .unwrap();
        assert!(layout.is_valid());
        assert_eq!(layout.root(), Some(host));
        assert!(!layout.contains(row));
        assert_eq!(layout.placement(a), Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }));
        assert_eq!(layout.placement(b), Some(DockSlot::Floating { host, index: 0 }));
        // The cascade-destroyed row is recorded in the inverse.
        assert!(o2.inverse().removed.iter().any(|dock| dock.id == row));

        o2.inverse().clone().replay_into(&mut layout);
        assert_eq!(layout, between);
        o1.inverse().clone().replay_into(&mut layout);
        assert_eq!(layout, before);
    }

    #[test]
    fn promote_host_on_an_existing_host_is_a_noop_success() {
        let (mut layout, host, _a, _b, _row) = host_a_b_row();
        let before = layout.clone();
        let anchor = DockingOperation::PromoteHost { dock: host }.apply(&mut layout).unwrap();
        let DockingOutcome::HostPromoted { dock, promoted, inverse } = &anchor else {
            panic!("expected HostPromoted");
        };
        assert_eq!(*dock, host);
        assert!(!*promoted);
        // No-op success: an identity inverse, replay restores nothing.
        assert!(inverse.containers.is_empty());
        assert!(inverse.removed.is_empty());
        assert!(inverse.created.is_empty());
        inverse.clone().replay_into(&mut layout);
        assert_eq!(layout, before);
    }

    #[test]
    fn promote_host_on_a_split_is_ineligible() {
        let (mut layout, _host, _a, _b, row) = host_a_b_row();
        let before = layout.clone();
        let err = DockingOperation::PromoteHost { dock: row }.apply(&mut layout).unwrap_err();
        assert_eq!(err, DockingError::IneligibleDock { id: row });
        assert_eq!(layout, before);
    }

    #[test]
    fn promote_host_is_persistent_and_round_trips_through_undo() {
        let mut layout = DockLayout::new();
        let p = layout.create_panel(Some("work".into())); // root
        let _ = DockingOperation::SetDockCollapsed { dock: p, collapsed: true }
            .apply(&mut layout)
            .unwrap();

        let anchor = DockingOperation::PromoteHost { dock: p }.apply(&mut layout).unwrap();
        let DockingOutcome::HostPromoted { dock, promoted, inverse } = &anchor else {
            panic!("expected HostPromoted");
        };
        assert_eq!(*dock, p);
        assert!(*promoted);
        // Persistent host, identity and content preserved; the panel
        // collapse preference does not carry over (hosts are not a
        // collapsible category in v1 — the dock lands expanded).
        assert_eq!(layout.get(p).unwrap().persistent(), Some(true));
        assert_eq!(layout.get(p).unwrap().collapsed(), None);
        assert_eq!(layout.get(p).unwrap().content(), Some(&"work".to_string()));
        assert!(layout.is_valid());

        inverse.clone().replay_into(&mut layout);
        // The promotion record restores the pre-promotion panel with
        // its collapse preference.
        assert_eq!(layout.get(p).unwrap().kind(), DockKind::Panel);
        assert_eq!(layout.get(p).unwrap().collapsed(), Some(true));
        assert_eq!(layout.get(p).unwrap().content(), Some(&"work".to_string()));
    }

    #[test]
    fn resize_region_requires_an_embed_host_and_an_occupied_region() {
        let (mut layout, host, _a, _b, row) = host_a_b_row();
        let before = layout.clone();

        // Not an embed host (the host-side `NotATabGroup` sibling).
        let err = DockingOperation::ResizeRegion {
            host: row,
            edge: EdgeRegion::Top,
            requested: 2.0,
            min: None,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::NotAnEmbedHost { id: row });

        // An unoccupied region has no boundary — and no preference —
        // to resize (weights live with the region lifecycle).
        let err = DockingOperation::ResizeRegion {
            host,
            edge: EdgeRegion::Top,
            requested: 2.0,
            min: None,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::RegionUnoccupied { host, edge: EdgeRegion::Top });
        assert_eq!(layout, before);
    }

    #[test]
    fn resize_region_rejects_invalid_requests_exclusively() {
        let (mut layout, host, a, _b, _row) = host_a_b_row();
        let _ = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Right),
        }
        .apply(&mut layout)
        .unwrap();
        let valid = layout.clone();

        // The sizing invariant is always an implicit bound.
        for junk in [0.0, f64::NAN, f64::INFINITY] {
            let err = DockingOperation::ResizeRegion {
                host,
                edge: EdgeRegion::Right,
                requested: junk,
                min: None,
            }
            .apply(&mut layout)
            .unwrap_err();
            assert_eq!(err, DockingError::RegionBoundViolation { host, edge: EdgeRegion::Right });
            assert_eq!(layout, valid); // rejected, never clamped
        }
        // A carried minimum is honored: a below-minimum request rejects.
        let err = DockingOperation::ResizeRegion {
            host,
            edge: EdgeRegion::Right,
            requested: 0.5,
            min: Some(1.0),
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::RegionBoundViolation { host, edge: EdgeRegion::Right });
        assert_eq!(layout, valid);

        // The bound endpoint itself is honored inclusively.
        let anchor = DockingOperation::ResizeRegion {
            host,
            edge: EdgeRegion::Right,
            requested: 1.0,
            min: Some(1.0),
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::RegionResized { edge, previous, .. } = &anchor else {
            panic!("expected RegionResized");
        };
        assert_eq!(*edge, EdgeRegion::Right);
        assert_eq!(*previous, 1.0);
    }

    #[test]
    fn resize_region_rewrites_one_weight_only_and_round_trips_through_undo() {
        let (mut layout, host, a, b, _row) = host_a_b_row();
        let _ = DockingOperation::Embed {
            source: InsertSource::Existing(a),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }
        .apply(&mut layout)
        .unwrap();
        let _ = DockingOperation::Embed {
            source: InsertSource::Existing(b),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Bottom),
        }
        .apply(&mut layout)
        .unwrap();
        assert!(layout.is_valid());
        let before = layout.clone();

        let anchor = DockingOperation::ResizeRegion {
            host,
            edge: EdgeRegion::Top,
            requested: 3.25,
            min: None,
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::RegionResized { host: h, edge, previous, inverse } = &anchor else {
            panic!("expected RegionResized");
        };
        assert_eq!(*h, host);
        assert_eq!(*edge, EdgeRegion::Top);
        assert_eq!(*previous, 1.0); // the embedding baseline
        // Exactly one stored weight changed: the occupant stays in its
        // region, the sibling region's preference is verbatim (the §9.5
        // coherence rule — no other placement invalidated).
        let regions = layout.get(host).unwrap().host_regions().unwrap();
        assert_eq!(regions.on(EdgeRegion::Top), Some(a));
        assert_eq!(regions.weight_on(EdgeRegion::Top), Some(3.25));
        assert_eq!(regions.on(EdgeRegion::Bottom), Some(b));
        assert_eq!(regions.weight_on(EdgeRegion::Bottom), Some(1.0));
        assert!(layout.is_valid());

        inverse.clone().replay_into(&mut layout);
        assert_eq!(layout, before);
    }

    // --- Floating (Phase 8, PRD §9.6, §9.3) ---

    #[test]
    fn set_floating_rect_round_trips_through_undo() {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(Some("host".into())); // auto-rooted, persistent
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        let r0 = FloatingRect { x: 0.125, y: 0.125, w: 0.25, h: 0.25 };
        let r1 = FloatingRect { x: 0.5, y: 0.5, w: 0.5, h: 0.5 };
        // Import-style surgery: floated entries on the host (the `Embed`
        // slot is the public route; this keeps the undo-replay fixture
        // minimal — the same reasoning as the region fixtures above).
        {
            let dock = layout.get_mut(host).expect("host exists");
            if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                floating.push(FloatingEntry::new(a, r0));
                floating.push(FloatingEntry::new(b, r1));
            }
        }
        assert!(layout.is_valid());
        let before = layout.clone();
        let requested = FloatingRect { x: 0.25, y: 0.25, w: 0.625, h: 0.5 };

        let outcome = DockingOperation::SetFloatingRect {
            host,
            index: 1,
            rect: requested,
            min_w: None,
            min_h: None,
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::FloatingRectSet { host: h, index, previous, inverse } = &outcome
        else {
            panic!("expected FloatingRectSet");
        };
        assert_eq!(*h, host);
        assert_eq!(*index, 1);
        assert_eq!(*previous, r1);
        // Exactly the named entry rewrote; the sibling entry and every
        // other placement stay verbatim.
        let floating = layout.get(host).unwrap().floating().unwrap();
        assert_eq!(floating[0].rect(), r0);
        assert_eq!(floating[1].rect(), requested);
        assert!(layout.is_valid());
        // The inverse owns the whole former collection: the entries —
        // dock *and* rect — restore together.
        assert_eq!(
            inverse.containers.first(),
            Some(&FormerCollection::Floating {
                host,
                entries: vec![FloatingEntry::new(a, r0), FloatingEntry::new(b, r1)],
            })
        );
        inverse.clone().replay_into(&mut layout);
        assert_eq!(layout, before);
    }

    #[test]
    fn embed_floating_initial_rect_validation() {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(Some("host".into()));
        assert!(layout.is_valid());
        let before = layout.clone();

        // An invalid initial rect rejects the embed (the §9.6
        // host-bounds placement invariant, the §9.3 rejection policy —
        // drop interactions neither clamp nor resolve), names the
        // would-be entry index, and leaves the layout untouched — no
        // residue even for a created source panel (the candidate is
        // discarded with the failure).
        for junk in [
            FloatingRect { x: 0.5, y: 0.0, w: f64::NAN, h: 0.5 },
            FloatingRect { x: -0.25, y: 0.0, w: 0.5, h: 0.5 },
            FloatingRect { x: 0.5, y: 0.0, w: 0.625, h: 0.5 }, // x + w > 1
            FloatingRect { x: 0.0, y: 0.0, w: 0.0, h: 0.5 },   // zero extent
            FloatingRect { x: 0.0, y: 0.0, w: 0.5, h: f64::INFINITY },
        ] {
            let err = DockingOperation::Embed {
                source: InsertSource::NewPanel { content: Some("f".into()) },
                host,
                slot: EmbedSlot::Floating { rect: Some(junk) },
            }
            .apply(&mut layout)
            .unwrap_err();
            assert_eq!(err, DockingError::FloatingBoundViolation { host, index: 0 });
            assert_eq!(layout, before);
        }

        // After a successful append the rejected embed names the
        // collection's append position (the would-be entry index).
        let _ = DockingOperation::Embed {
            source: InsertSource::NewPanel { content: Some("f1".into()) },
            host,
            slot: EmbedSlot::Floating { rect: None },
        }
        .apply(&mut layout)
        .unwrap();
        assert!(layout.is_valid());

        // An existing source is equally untouched by a rejected initial
        // rect: the pre-check runs before any detach or mutation.
        let region = DockingOperation::Embed {
            source: InsertSource::NewPanel { content: Some("p1".into()) },
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }
        .apply(&mut layout)
        .unwrap();
        let DockingOutcome::Embedded { source: p1, .. } = &region else {
            panic!("expected Embedded");
        };
        let p1 = *p1;
        let before = layout.clone();
        let err = DockingOperation::Embed {
            source: InsertSource::Existing(p1),
            host,
            slot: EmbedSlot::Floating {
                rect: Some(FloatingRect { x: 0.5, y: 0.0, w: 0.625, h: 0.5 }),
            },
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::FloatingBoundViolation { host, index: 1 });
        assert_eq!(layout, before); // p1 still in the region, no residue
        assert_eq!(
            layout.get(host).unwrap().host_regions().unwrap().on(EdgeRegion::Top),
            Some(p1)
        );
        assert!(layout.is_valid());
    }
}
