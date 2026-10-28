//! The docking layout model: root, docks, placements, and traversal.
//!
//! [`DockLayout`] owns every [`Dock`] keyed by stable [`DockId`]s (see
//! `dock.rs`) plus the designated root (PRD DM-06). It is a headless model:
//! no DOM, no CSS, no pointer state, and no rendering coordinates are
//! required to represent, inspect, or change a layout (PRD G-01).
//!
//! ## Model decisions (established in Phase 2)
//!
//! - **Root.** The root is the current top-level dock; it is
//!   `Some(id)` for a non-empty layout and `None` for the explicit
//!   empty layout (PRD §7.4). The root is not itself placed anywhere;
//!   every other dock occupies exactly one [`DockSlot`]. The root may be
//!   any category. Creation operations prefer structural splits
//!   (per the design proposal §3.6); this is operation policy, not a
//!   structural restriction, so single-panel and tab-rooted layouts are
//!   representable without fake containers (PRD §7.4).
//! - **Ownership.** A dock's placement is *derived* by scan from the
//!   containers that reference it (split children, tab members, host
//!   regions, host floating lists). Containers' collections are the single
//!   source of truth — no parent back-pointers, hence no desync between a
//!   dock's stored parent and its real owner. In a validated layout the
//!   scan owns exactly one parent per child.
//! - **Single identity scope.** `next_id` is never reset by
//!   [`clear`](Self::clear); ids keep identity after structural changes
//!   (precedent: `Canvas`, `Layers`).
//! - **Creation vs. mutation.** The creation methods below build
//!   structurally consistent pieces (operation support primitives).
//!   Re-arranging existing docks — detach/reattach, split/insert,
//!   tab grouping, embedding, floating, removal, normalization — is
//!   expressed as [operations](super::operations) (Phase 4) and must keep
//!   the layout valid on failure (PRD §8.1, §8.2).
//!
//! ## What this environment deliberately does *not* own
//!
//! - Panel rendering, headers, tabs, splitters, floating windows
//!   (Phase 11 / frontend).
//! - Pointer hit-testing, drag state, drop overlays (frontend).
//! - Actual pixel geometry; weights and constraints are logical state
//!   derived by the frontend (PRD §9.1).
//! - Application-owned panel content: preserved as an opaque
//!   [`ContentRef`] only (PRD §7.3).

use std::collections::{HashMap, VecDeque};

use super::dock::{ContentRef, Dock, DockId, DockKind, DockRole, SplitOrientation};
use super::error::DockingError;
use super::sizing::SplitSizes;

/// Headless docking layout.
///
/// A map of docks plus a designated root. Every dock except the root
/// occupies a valid [`DockSlot`](super::slot::DockSlot); collections
/// owned by containers are the single source of structural truth.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DockLayout {
    docks: HashMap<DockId, Dock>,
    root: Option<DockId>,
    next_id: u64,
}

impl DockLayout {
    /// Create the explicit empty layout: no docks, no root.
    ///
    /// This is the valid empty-layout representation (PRD §7.4); it must
    /// not require a fake panel (PRD §7.4 "MUST NOT create a fake
    /// user-facing panel").
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of docks in the layout (excluding nothing; the root is a
    /// dock too).
    pub fn len(&self) -> usize {
        self.docks.len()
    }

    /// `true` when the layout contains no docks and no root.
    pub fn is_empty(&self) -> bool {
        self.docks.is_empty() && self.root.is_none()
    }

    /// The designated root's id, if any.
    pub fn root(&self) -> Option<DockId> {
        self.root
    }

    /// The designated root dock, if any.
    pub fn root_dock(&self) -> Option<&Dock> {
        self.root.and_then(|id| self.docks.get(&id))
    }

    /// Whether a dock with `id` exists.
    pub fn contains(&self, id: DockId) -> bool {
        self.docks.contains_key(&id)
    }

    /// Look up a dock by id.
    pub fn get(&self, id: DockId) -> Option<&Dock> {
        self.docks.get(&id)
    }

    /// Mutable lookup by id — `pub(crate)` so that only environment code
    /// (operations) can mutate docks directly; external mutation must not
    /// bypass operation validation (PRD §15.3).
    pub(crate) fn get_mut(&mut self, id: DockId) -> Option<&mut Dock> {
        self.docks.get_mut(&id)
    }

    /// Insert or replace an existing dock (for operation undo paths).
    ///
    /// Restores the dock with its original id; raises the id counter if
    /// needed (precedent: `Canvas::insert_object`, `Layers::insert_layer`).
    pub(crate) fn put_dock(&mut self, dock: Dock) {
        let id = dock.id;
        if id.as_raw() >= self.next_id {
            self.next_id = id.as_raw() + 1;
        }
        self.docks.insert(id, dock);
    }

    /// Remove a dock and return it (for operation undo paths).
    ///
    /// The caller is an operation; it is responsible for detaching the
    /// dock from its placement first (PRD §8.5).
    pub(crate) fn take_dock(&mut self, id: DockId) -> Option<Dock> {
        self.docks.remove(&id)
    }

    /// Designate (or clear) the root without the `set_root`
    /// preconditions. `pub(crate)`: operation pipelines build transient
    /// states (a detached root while the candidate is being restructured)
    /// and undo replay restores the root verbatim; external code cannot
    /// bypass the operation contract (PRD §15.3).
    pub(crate) fn set_root_unchecked(&mut self, root: Option<DockId>) {
        self.root = root;
    }

    /// Restore one container collection from a [`FormerCollection`]
    /// record (undo replay). `pub(crate)`: inverse-data restoration, like
    /// `put_dock` / `take_dock`.
    pub(crate) fn restore_collection(&mut self, record: &super::operations::FormerCollection) {
        use super::operations::FormerCollection;
        match record {
            FormerCollection::Split { id, children, sizes } => {
                if let Some(dock) = self.get_mut(*id) {
                    if let DockRole::Split { children: c, sizes: s, .. } = &mut dock.role {
                        *c = children.clone();
                        *s = sizes.clone();
                    }
                }
            }
            FormerCollection::Tab { id, members, active } => {
                if let Some(dock) = self.get_mut(*id) {
                    if let DockRole::TabGroup { members: m, active: a, .. } = &mut dock.role {
                        *m = members.clone();
                        *a = *active;
                    }
                }
            }
            FormerCollection::Region { host, edge, previous, weight } => {
                if let Some(dock) = self.get_mut(*host) {
                    if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                        // The occupant and its sizing weight restore
                        // together; the weight lifecycle is handled by
                        // `assign` (a cleared slot resets to the embed
                        // baseline).
                        regions.assign(*edge, *previous, weight.unwrap_or(1.0));
                    }
                }
            }
            FormerCollection::Floating { host, entries } => {
                if let Some(dock) = self.get_mut(*host) {
                    if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                        // The entry's dock and its logical rect restore
                        // together (the record snapshots full entries —
                        // the Region record's occupant-and-preference
                        // precedent, PRD §9.6).
                        *floating = entries.clone();
                    }
                }
            }
            FormerCollection::Panel { id, collapsed } => {
                if let Some(dock) = self.get_mut(*id) {
                    if let DockRole::Panel { collapsed: c, .. } = &mut dock.role {
                        *c = *collapsed;
                    }
                }
            }
            FormerCollection::Promotion { id, collapsed } => {
                // Undo of a promotion: the promoted panel returns to
                // its pre-op payload. Content flows through promotion
                // unchanged (the environment never rewrites it), so the
                // record carries the collapse preference only.
                if let Some(dock) = self.get_mut(*id) {
                    let content = dock.content().cloned();
                    dock.role = DockRole::Panel { content, collapsed: *collapsed };
                }
            }
            FormerCollection::Demotion { id } => {
                // Undo of a demotion: the demoted dock becomes an empty
                // temporary host again — the exact pre-demotion role
                // (demotion only fires on hosts whose regions and
                // floating collection are empty, and the weight
                // lifecycle keeps empty regions at the baseline).
                if let Some(dock) = self.get_mut(*id) {
                    let content = dock.content().cloned();
                    dock.role = DockRole::EmbedHost {
                        content,
                        regions: super::embed_host::HostRegions::new(),
                        floating: Vec::new(),
                        persistent: false,
                    };
                }
            }
        }
    }

    /// Iterate over all docks. Order is unspecified (HashMap order); use
    /// [`placement`](Self::placement)-based or traversal methods when
    /// order matters.
    pub fn iter(&self) -> impl Iterator<Item = &Dock> {
        self.docks.values()
    }

    // --- Creation (operation support primitives) ---

    fn alloc_id(&mut self) -> DockId {
        let id = DockId::from_raw(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// Create a panel dock with the given content reference.
    ///
    /// When the layout is completely empty, the panel becomes the root;
    /// otherwise it is created unplaced and must be placed by an
    /// operation (PRD DM-07). Auto-rooting never fires for a transient
    /// rootless state (a mid-operation build), keeping creation
    /// placement-predictable. The panel starts expanded; the collapse
    /// preference is a `SetDockCollapsed` operation (PRD §9.7).
    pub fn create_panel(&mut self, content: Option<ContentRef>) -> DockId {
        let id = self.alloc_id();
        let role = DockRole::Panel { content, collapsed: false };
        self.docks.insert(id, Dock::new(id, role));
        if self.docks.len() == 1 {
            self.root = Some(id);
        }
        id
    }

    /// Create an embed host with the given underlying work area and
    /// empty edge regions / floating collection.
    ///
    /// Root rule as in [`create_panel`](Self::create_panel). Created
    /// hosts are **persistent** (PRD §8.9): an explicitly configured
    /// host remains a host even when empty; temporary hosts exist only
    /// through promotion (see `PromoteHost` / `Embed`).
    pub fn create_embed_host(&mut self, content: Option<ContentRef>) -> DockId {
        let id = self.alloc_id();
        let role = DockRole::EmbedHost {
            content,
            regions: super::embed_host::HostRegions::new(),
            floating: Vec::new(),
            persistent: true,
        };
        self.docks.insert(id, Dock::new(id, role));
        if self.docks.len() == 1 {
            self.root = Some(id);
        }
        id
    }

    /// Create a split container over previously unplaced docks.
    ///
    /// Preconditions (all errors leave the layout unchanged):
    /// - `children` has at least two docks ([`DockingError::InsufficientChildren`]);
    /// - every child exists ([`DockingError::UnknownReference`]);
    /// - children are unique ([`DockingError::DuplicateMembership`]);
    /// - every child is currently unplaced ([`DockingError::AlreadyPlaced`]);
    /// - `sizes` has one weight per child
    ///   ([`DockingError::SizeCountMismatch`]).
    ///
    /// The created split is unplaced; designate it with
    /// [`set_root`](Self::set_root) if needed.
    pub fn create_split(
        &mut self,
        orientation: SplitOrientation,
        children: Vec<DockId>,
        sizes: SplitSizes,
    ) -> Result<DockId, DockingError> {
        if children.len() < 2 {
            return Err(DockingError::InsufficientChildren {
                minimum: 2,
                found: children.len(),
            });
        }
        if !sizes.matches(children.len()) {
            return Err(DockingError::SizeCountMismatch {
                sizes: sizes.len(),
                children: children.len(),
            });
        }
        // Existence, uniqueness, and unplaced checks (none of these mutate).
        for &child in &children {
            if !self.docks.contains_key(&child) {
                return Err(DockingError::UnknownReference { id: child });
            }
        }
        if let Some(dup) = find_duplicate(children.iter().copied()) {
            return Err(DockingError::DuplicateMembership { id: dup });
        }
        self.check_all_unplaced(&children)?;

        let id = self.alloc_id();
        let role = DockRole::Split {
            orientation,
            children,
            sizes,
        };
        self.docks.insert(id, Dock::new(id, role));
        Ok(id)
    }

    /// Create a tab group over previously unplaced docks.
    ///
    /// Preconditions as in [`create_split`](Self::create_split) except
    /// that a group needs at least one member, and `active` must index
    /// `members` ([`DockingError::ActiveIndexOutOfRange`]).
    pub fn create_tab_group(
        &mut self,
        members: Vec<DockId>,
        active: usize,
    ) -> Result<DockId, DockingError> {
        if members.is_empty() {
            return Err(DockingError::InsufficientChildren {
                minimum: 1,
                found: 0,
            });
        }
        if active >= members.len() {
            return Err(DockingError::ActiveIndexOutOfRange {
                index: active,
                member_count: members.len(),
            });
        }
        for &member in &members {
            if !self.docks.contains_key(&member) {
                return Err(DockingError::UnknownReference { id: member });
            }
        }
        if let Some(dup) = find_duplicate(members.iter().copied()) {
            return Err(DockingError::DuplicateMembership { id: dup });
        }
        self.check_all_unplaced(&members)?;

        let id = self.alloc_id();
        let role = DockRole::TabGroup { members, active };
        self.docks.insert(id, Dock::new(id, role));
        Ok(id)
    }

    fn check_all_unplaced(&self, candidates: &[DockId]) -> Result<(), DockingError> {
        for &candidate in candidates {
            if Some(candidate) == self.root || self.find_slot_in_containers(candidate).is_some() {
                return Err(DockingError::AlreadyPlaced { id: candidate });
            }
        }
        Ok(())
    }

    /// Designate or clear the root.
    ///
    /// - `Some(id)`: the layout must not already have a root
    ///   ([`DockingError::RootAlreadySet`]); `id` must exist
    ///   ([`DockingError::UnknownReference`]) and be unplaced
    ///   ([`DockingError::AlreadyPlaced`]). Re-rooting an existing layout
    ///   is an operation concern (Phase 4): clear the root explicitly
    ///   first, so the intermediate state is visible to the operation.
    /// - `None`: always succeeds; docks (if any) remain, placed as before.
    ///   An empty root is a transient mid-operation state; layout
    ///   validation (Phase 3) rejects a *persistent* empty root with docks.
    pub fn set_root(&mut self, id: Option<DockId>) -> Result<(), DockingError> {
        match id {
            None => {
                self.root = None;
                Ok(())
            }
            Some(id) => {
                if let Some(current) = self.root {
                    return Err(DockingError::RootAlreadySet { current });
                }
                if !self.docks.contains_key(&id) {
                    return Err(DockingError::UnknownReference { id });
                }
                if self.find_slot_in_containers(id).is_some() {
                    return Err(DockingError::AlreadyPlaced { id });
                }
                self.root = Some(id);
                Ok(())
            }
        }
    }

    /// Install an unplaced split container over the given children and
    /// sizes (operation-pipeline helper, like `put_dock`/`take_dock`).
    ///
    /// Unlike `create_split` (public creation) this does **not** require
    /// the children to be unplaced: the wrap case re-parents a currently
    /// placed target into the fresh container, the operation pipeline
    /// owns and validates those preconditions itself (PRD §8.6), and the
    /// candidate gate (`validate`) checks the committed result.
    pub(crate) fn put_split(
        &mut self,
        orientation: SplitOrientation,
        children: Vec<DockId>,
        sizes: SplitSizes,
    ) -> DockId {
        let id = self.alloc_id();
        self.docks
            .insert(id, Dock::new(id, DockRole::Split { orientation, children, sizes }));
        id
    }

    /// Install an unplaced tab group over the given members (operation
    /// helper, see `put_split`).
    pub(crate) fn put_tab_group(&mut self, members: Vec<DockId>, active: usize) -> DockId {
        let id = self.alloc_id();
        self.docks
            .insert(id, Dock::new(id, DockRole::TabGroup { members, active }));
        id
    }

    /// Remove all docks. The id counter is **not** reset
    /// (precedent: `Canvas::clear`, `Layers::clear`).
    pub fn clear(&mut self) {
        self.docks.clear();
        self.root = None;
    }

    // --- Placement inspection ---

    /// The slot a dock currently occupies.
    ///
    /// Returns `None` when the dock does not exist in the layout.
    /// A missing placement for an *existing* dock (an orphan) is a
    /// transient mid-operation state that layout validation (Phase 3)
    /// rejects.
    pub fn placement(&self, id: DockId) -> Option<super::slot::DockSlot> {
        use super::slot::DockSlot;
        if Some(id) == self.root {
            return Some(DockSlot::Root);
        }
        self.find_slot_in_containers(id)
    }

    /// Search containers for the slot referencing `id`.
    ///
    /// Scan order (for the trivially-rare multi-owner mid-flight states):
    /// root first, then the map's split containers and tab groups, then
    /// hosts. In a validated layout at most one owner exists.
    fn find_slot_in_containers(
        &self,
        id: DockId,
    ) -> Option<super::slot::DockSlot> {
        use super::slot::DockSlot;
        for (parent_id, dock) in &self.docks {
            match &dock.role {
                DockRole::Split { children, .. } => {
                    if let Some(index) = children.iter().position(|c| *c == id) {
                        return Some(DockSlot::SplitChild { parent: *parent_id, index });
                    }
                }
                DockRole::TabGroup { members, .. } => {
                    if let Some(index) = members.iter().position(|m| *m == id) {
                        return Some(DockSlot::TabMember { parent: *parent_id, index });
                    }
                }
                DockRole::EmbedHost { regions, floating, .. } => {
                    if let Some(edge) = regions.owner_of(id) {
                        return Some(DockSlot::HostEdge { host: *parent_id, edge });
                    }
                    if let Some(index) = floating.iter().position(|entry| entry.dock == id) {
                        return Some(DockSlot::Floating { host: *parent_id, index });
                    }
                }
                DockRole::Panel { .. } => {}
            }
        }
        None
    }

    /// Number of placements the dock `id` currently occupies: `1` when it
    /// is the root or referenced by exactly one container slot, `0` when
    /// unplaced (an orphan), `2+` when multiply claimed — including a
    /// dock listed twice in one collection.
    ///
    /// `pub(crate)`: whole-layout invariant accounting used by
    /// [validation](super::validation) (PRD DM-07); external code
    /// inspects placement via [`placement`](Self::placement).
    pub(crate) fn placement_count(&self, id: DockId) -> usize {
        let mut count = usize::from(Some(id) == self.root);
        for dock in self.docks.values() {
            match &dock.role {
                DockRole::Split { children, .. } => {
                    count += children.iter().filter(|c| **c == id).count();
                }
                DockRole::TabGroup { members, .. } => {
                    count += members.iter().filter(|m| **m == id).count();
                }
                DockRole::EmbedHost { regions, floating, .. } => {
                    count += usize::from(regions.top == Some(id))
                        + usize::from(regions.bottom == Some(id))
                        + usize::from(regions.left == Some(id))
                        + usize::from(regions.right == Some(id));
                    count += floating.iter().filter(|entry| entry.dock == id).count();
                }
                DockRole::Panel { .. } => {}
            }
        }
        count
    }

    // --- Traversal and hierarchy queries ---
    //
    // All reference edges are visited (split children, tab members, host
    // regions in fixed top/bottom/left/right order, then floating order),
    // so every supported placement category participates in lookup and
    // validation (PRD §7.7).

    /// Depth-first reachability check: is `maybe_descendant` reachable
    /// from `ancestor` (or the same dock)? Host regions and floating
    /// entries count as containment for cycle purposes (PRD §10.4).
    pub fn is_descendant(&self, maybe_descendant: DockId, ancestor: DockId) -> bool {
        if maybe_descendant == ancestor {
            return true;
        }
        let mut queue = VecDeque::new();
        queue.push_back(ancestor);
        while let Some(current) = queue.pop_front() {
            if let Some(dock) = self.docks.get(&current) {
                for child in dock.structural_children() {
                    if child == maybe_descendant {
                        return true;
                    }
                    queue.push_back(child);
                }
            }
        }
        false
    }

    /// All docks reachable from `id` via structural references
    /// (not including `id` itself), in BFS order. Every placement
    /// category is traversed, not only ordinary container children.
    pub fn descendants(&self, id: DockId) -> Vec<DockId> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(id);
        while let Some(current) = queue.pop_front() {
            if let Some(dock) = self.docks.get(&current) {
                for child in dock.structural_children() {
                    result.push(child);
                    queue.push_back(child);
                }
            }
        }
        result
    }

    /// The owner chain of `id`, root first, ending at `id` itself.
    /// Empty when `id` is unknown. The topmost entry is the root (unless
    /// mid-flight orphaning exists).
    pub fn path(&self, id: DockId) -> Vec<DockId> {
        if !self.docks.contains_key(&id) {
            return Vec::new();
        }
        let mut chain = vec![id];
        loop {
            let owner = self
                .find_slot_in_containers(*chain.last().unwrap())
                .and_then(|slot| slot.owner());
            match owner {
                Some(parent) => chain.push(parent),
                None => break,
            }
        }
        chain.reverse(); // root first, leaf last
        chain
    }

    /// Maximum nesting depth (longest root-to-leaf chain, counting host
    /// regions and floating entries as nesting edges). Zero for the
    /// empty layout.
    pub fn depth(&self) -> usize {
        match self.root {
            Some(id) => self.subtree_height(id),
            None => 0,
        }
    }

    fn subtree_height(&self, id: DockId) -> usize {
        match self.docks.get(&id) {
            Some(dock) => {
                1 + dock
                    .structural_children()
                    .into_iter()
                    .map(|child| self.subtree_height(child))
                    .max()
                    .unwrap_or(0)
            }
            None => 0,
        }
    }

    /// All dock ids in the layout.
    pub fn all_docks(&self) -> Vec<DockId> {
        self.docks.keys().copied().collect()
    }

    /// The kind of a dock, if it exists.
    pub fn kind_of(&self, id: DockId) -> Option<DockKind> {
        self.docks.get(&id).map(Dock::kind)
    }
}

/// Find a duplicate value in a sequence, if any.
/// Shared by creation validation.
fn find_duplicate(values: impl Iterator<Item = DockId>) -> Option<DockId> {
    let mut seen: Vec<DockId> = Vec::new();
    for value in values {
        if seen.contains(&value) {
            return Some(value);
        }
        seen.push(value);
    }
    None
}
