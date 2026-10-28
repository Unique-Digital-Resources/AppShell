//! # Docking Environment
//!
//! Headless docking-layout model. Provides dock identity, structural
//! categories (panel, split container, tab group, embed host), placement
//! slots, and hierarchy traversal — without imposing any UI, pointer
//! input, drop overlays, or rendering (PRD G-01, AC-16).
//!
//! ## What this environment owns
//!
//! - Stable, opaque dock identity via [`DockId`].
//! - [`Dock`] / [`DockRole`] — the five dock categories from the design
//!   baseline expressed as four model categories: panel, split
//!   (orientation `Row`/`Column`), tab group, embed host
//!   (PRD §7.2).
//! - A [`DockLayout`] container that owns all docks, a designated root,
//!   and derived placement inspection ([`DockSlot`] / [`EdgeRegion`]).
//! - Logical sizing state for split children via [`SplitSizes`]
//!   (relative weights, stored verbatim; the representation decision is
//!   documented in [`sizing`]; PRD §9.2 baseline).
//! - Splitter resizing and logical collapse preferences (PRD
//!   §9.3–§9.4, §9.7): [`DockingOperation::ResizeSiblings`] rewrites the
//!   adjacent split-child pair with conserved pair total and
//!   frontend-translated minimum bounds (rejection policy),
//!   [`DockingOperation::SetDockCollapsed`] carries the panel collapse
//!   flag without touching structure or allocation.
//! - Whole-layout invariant checks via [`DockLayout::validate`] /
//!   [`validation`](validation) (PRD §7.7; imported-state gate).
//! - Deterministic structural normalization via
//!   [`DockLayout::normalize`] ([`normalization`]): containers whose
//!   membership fell below their steady-state minimum are resolved
//!   along documented rules — the shared pass of the operation
//!   pipeline (mutate → normalize → validate → commit; PRD §8.12).
//! - Value-type structural operations ([`DockingOperation`] /
//!   [`DockingOutcome`], see [`operations`]): insert-beside, tab
//!   creation/join/reorder/activation, removal, splitter resize,
//!   collapse preference — candidate-validated and atomic, carrying
//!   full inverse data ([`UndoData`]) for engine undo
//!   (PRD §8.1–§8.7, §9.3–§9.4, §9.7, OP-01…OP-04).
//! - Environment-specific [`DockingError`] (PRD §14 naming/style).
//! - Embedding and the host lifecycle (PRD §8.8–§8.9):
//!   [`DockingOperation::Embed`] places a source in a host's edge
//!   region ([`EmbedSlot::Region`]) or floating collection
//!   ([`EmbedSlot::Floating`]), auto-promoting a panel target the way
//!   §8.9 sanctions; [`DockingOperation::PromoteHost`] is the explicit,
//!   *persistent* promotion. Hosts are persistent (created via
//!   `CreateEmbedHost` or by explicit promotion) or temporary
//!   promotions, distinguished by `Dock::persistent`; an emptied
//!   temporary host demotes back through the shared normalization
//!   pass. Removing a host removes its whole arrangement.
//! - Persistent region sizing (PRD §9.5): every occupied edge region
//!   carries a verbatim slot-attached weight; [`DockingOperation::ResizeRegion`]
//!   rewrites exactly one region (absolute drift-free requests, carried
//!   minimum bounds, rejection policy).
//! - Floating docking structures (PRD §9.6, Phase 8):
//!   [`DockingOperation::Embed`] floats any source subtree into a
//!   host's floating collection — host-anchored, one entry per floated
//!   root, internal structure preserved — carrying the entry's initial
//!   [`FloatingRect`] (fractions of the host's body, the PRD §9.6
//!   decided representation: the dock body rectangle, contained in the
//!   host's logical bounds); [`DockingOperation::SetFloatingRect`]
//!   rewrites exactly one entry's rect (absolute drift-free requests,
//!   carried minimum bounds, rejection policy). Re-docking a floater is
//!   the source side of the same ops (`Embed` region, `InsertBeside`,
//!   `TabJoin`), and a floating dock as a directional target stays
//!   rejected (`NotSplittable`); visual stacking is frontend-derived
//!   (PRD §11.3). The state types and lifecycle docs live in
//!   [`floating`].
//!
//! ## What this environment deliberately does *not* own
//!
//! - Application panel content. Docks carry an opaque
//!   `ContentRef` reference only; hosts own the real
//!   content (PRD §7.3).
//! - Floating-dock window decoration and visual stacking (z-order) are
//!   frontend-derived; the environment stores the entry's logical
//!   body rect only (PRD §9.6, §11.3).
//! - Execution infrastructure (commands, history, events). Phase 9 will
//!   wrap operations in Domain Engine execution like the existing
//!   environments.
//! - Rendering, input, geometry measurement, drag-and-drop. The
//!   interaction contract that hands these off to the frontend is
//!   specified below (PRD §10–§11).
//!
//! ## Model notes
//!
//! - The layout is a hierarchical structure for ordinary split and tab
//!   relationships with explicit, non-tree placement collections for host
//!   edge regions and floating docks (PRD §7.6).
//! - The root is the current top-level dock, `None` for the explicit
//!   empty layout; normalization never removes it (PRD DM-06, §8.12).
//! - Domain engines attach their own data externally, keyed by [`DockId`]
//!   (precedent: Canvas2D / Layers).
//!
//! ## Frontend handoff — the interaction contract (PRD §10, §11)
//!
//! This environment implements no drag-and-drop. A frontend drives it
//! with **intent + eligibility + commit** and never pointer coordinates:
//! hit-testing, pointer tracking, drop overlays, visual previews,
//! animations, and window management stay with the frontend (PRD §10,
//! §11.3). The contract below maps the frontend's drag lifecycle onto
//! the existing operation and read API — it adds **no** operation, **no**
//! eligibility policy, and **no** persistent interaction state: the
//! domain needs no drag session (the plan's separate interaction /
//! drag-session file is deliberately not added, PRD §11.2), so purely
//! preview state cannot enter the committed layout — nor the persisted
//! form (the persisted payload is the committed model; the Phase-10
//! restore gate would decline anything else).
//!
//! Every request is a plain [`DockingOperation`] value submitted
//! through the engine (`DomainEngine::execute_docking` — real
//! undo/redo, transactions, per-operation events), or applied directly
//! on a [`DockLayout`] for host-controlled flows. The stages need no
//! particular event framework (PRD §10.1): a drag is only ever observed
//! here as the one confirmed request it commits.
//!
//! ### Stage-by-stage lifecycle (PRD §10.1)
//!
//! | Frontend stage | How it is served |
//! |---|---|
//! | Identify the dragged element or subtree | the source dock's [`DockId`], or [`InsertSource::NewPanel`] for content not yet in the layout — moves keep identity and whole subtrees intact |
//! | Begin a drag | *frontend-only state* — nothing to call; the model cannot observe a drag |
//! | Determine eligible targets | the read-side discovery table below — advisory (a frontend SHOULD avoid presenting invalid targets); domain validation stays authoritative (PRD §10.4) |
//! | Identify the proposed target + drop region | the target [`DockId`] and the user's region intent; [`DockLayout::placement`] reflects the target's current slot |
//! | Determine the docking mode | the frontend's mode selection; mode + region together select the operation (PRD §10.2–§10.3) |
//! | Commit the confirmed drop | submit the operation value — one confirmed drop is one request |
//! | Display the result / report a rejection | [`DockingOutcome`] on success (the realized ids and slots); [`DockingError`] on rejection, naming what to re-scope |
//! | Clear transient interaction state | *frontend-only* — happens on success, rejection, and cancellation alike; no domain call is involved |
//!
//! ### Region + mode → operation (PRD §10.2–§10.3)
//!
//! A region says *where*; mode + region together determine the request
//! (PRD §10.2). The five conceptual regions map to the three docking
//! modes as follows — and an unsupported combination expects the
//! frontend to choose another mode; a submitted-but-invalid request is
//! rejected as its own value, never silently executed as a different
//! operation (PRD §10.3):
//!
//! | Docking mode | Edge intent (top / bottom / left / right) | Center intent |
//! |---|---|---|
//! | Split/insert | [`DockingOperation::InsertBeside`] with [`SplitDirection`] (a new or compatible row/column limb beside the target) | not a split intent — submit the tab or embed mode instead |
//! | Tab grouping | not a tab intent | [`DockingOperation::TabJoin`] (joins the target's group, or wraps the panel in a new group inheriting its slot) |
//! | Embedding | [`DockingOperation::Embed`] with [`EmbedSlot::Region`] ([`EdgeRegion`]) | a drop released on the host's body — outside any edge zone: `Embed` with [`EmbedSlot::Floating`], whose `rect` carries the drop interaction's logical placement (`None` = the floating baseline, PRD §9.6) |
//!
//! The same requests work from keyboard, touch, or programmatic
//! callers: the vocabulary carries intent, never an input method
//! (PRD §10.6).
//!
//! ### Eligibility discovery (PRD §10.4)
//!
//! Advisory pre-checks from the public read API — enough to avoid
//! presenting invalid targets; a check that misses (or disagrees with
//! commit) is resolved by the commit itself, which re-validates every
//! precondition:
//!
//! | Interface question | Read-side answer |
//! |---|---|
//! | Capable embed target (a host — or a **panel**, the `Embed` auto-promotion candidate, PRD §8.9) | [`Dock::kind`] is `EmbedHost` or `Panel` |
//! | Edge region free? | [`Dock::host_regions`] + `HostRegions::on(edge)` is `None` |
//! | Directional insert defined for the target? | [`DockLayout::placement`] is not [`DockSlot::Floating`] |
//! | Tab target / source joinable (v1 membership is panels only) | [`Dock::kind`] is `Panel` for both |
//! | Cycle guard — is the would-be host inside the dragged subtree? | [`DockLayout::is_descendant`] (self-inclusive) |
//!
//! Whatever escapes the pre-filter is still rejected at commit —
//! frontend checks are not a replacement for domain validation
//! (PRD §10.4). The settled rejections carry actionable payloads so the
//! drag can be re-scoped:
//!
//! | Rejected intent | [`DockingError`] | Carries |
//! |---|---|---|
//! | Unknown source or target id | `DockNotFound` | the offending id |
//! | Dropped onto itself or its own subtree | `CycleDetected` | the offending target/host id |
//! | Embed into a split / tab-group target | `IneligibleDock` | the target id |
//! | Embed onto an occupied edge region | `RegionOccupied` | the host + region blocked |
//! | Directional-insert beside a floated dock | `NotSplittable` | the floated dock's id |
//! | Non-eligible argument category (`NotATabGroup` / `NotASplit` / `NotAnEmbedHost`) | named variant | the offending id |
//! | Out-of-range index (`TabIndexOutOfRange` / `SplitChildIndexOutOfRange` / `FloatingIndexOutOfRange`) | named variant | index + current count |
//! | Resize would violate a carried minimum or the sizing/rect invariant — never clamped | `SizeBoundViolation` / `RegionBoundViolation` / `FloatingBoundViolation` | the offending region |
//!
//! A rejected request is the *whole* consequence: the layout is
//! untouched exactly as it was (candidate-clone atomicity), and no
//! history entry or event is recorded for it (PRD §10.5, §8.2).
//!
//! ### Gesture commit boundaries (PRD §11.4)
//!
//! Resize / floating-move gestures commit per op with the
//! absolute-request contract documented on each operation (see
//! [`operations`]); intermediate gesture measurements stay frontend
//! state. Committed frames live in history like any operation, and the
//! frontend chooses its commit boundary at gesture end:
//!
//! - **Per-frame rollback**: frames without a wrapper are single
//!   history entries; walked back one `undo()` call at a time.
//! - **Gesture-span transaction**: wrap the frames in a transaction —
//!   `rollback_transaction()` reverts the whole span in one call, and a
//!   committed span undoes/redoes as one unit.
//!
//! Cancellation *before* a commit is simply not submitting; there is no
//! domain-side "abort".

pub mod dock;
pub mod embed_host;
pub mod error;
pub mod floating;
pub mod model;
pub mod normalization;
pub mod operations;
pub mod slot;
pub mod sizing;
pub mod validation;

pub use dock::{ContentRef, Dock, DockId, DockKind, DockRole, SplitOrientation};
pub use embed_host::HostRegions;
pub use error::DockingError;
pub use floating::{FloatingEntry, FloatingRect};
pub use model::DockLayout;
pub use operations::{
    DockingOperation, DockingOutcome, EmbedSlot, FormerCollection, InsertSource, SplitDirection,
    UndoData,
};
pub use slot::{DockSlot, EdgeRegion};
pub use sizing::SplitSizes;
