//! Docking domain operations: wrapper around Docking operations that
//! carries undo/redo information and integrates with App Engine history
//! and events.
//!
//! Real undo, the [`ExecutedCanvasOperation`](super::canvas_operations::ExecutedCanvasOperation)
//! precedent (Phase-1 review R-A1): [`undo`](ExecutedDockingOperation::undo)
//! replays the outcome's [`UndoData`] through the `pub(crate)`
//! `replay_into` seam every outcome carries. [`redo`](ExecutedDockingOperation::redo)
//! replays a *forward* record — the mirror of the inverse, built from the
//! committed result at execution time — because re-applying the operation
//! is **not** id-preserving: [`DockId`]s are counter-issued and never
//! renumbered, so re-running an id-allocating operation after an exact
//! undo would allocate fresh ids and diverge from every recorded id in
//! the surrounding history; see `ExecutedDockingOperation::redo_record`.

use crate::app_engine::HistoryEntry;
use crate::domain_foundation::environments::docking::{
    Dock, DockId, DockLayout, DockRole, DockingError, DockingOperation, DockingOutcome,
    FormerCollection, UndoData,
};

#[derive(Debug, Clone)]
pub struct ExecutedDockingOperation {
    operation: DockingOperation,
    outcome: DockingOutcome,
    /// The forward replay record (pre→post mirror) for
    /// [`redo`](ExecutedDockingOperation::redo).
    redo: UndoData,
}

impl ExecutedDockingOperation {
    /// Execute the operation and capture the executed entry.
    ///
    /// The operation goes through the environment pipeline exactly once
    /// (mutate → normalize → validate → commit); the committed post-state
    /// and the outcome's inverse data yield the forward record below.
    pub fn execute(
        layout: &mut DockLayout,
        operation: DockingOperation,
    ) -> Result<Self, DockingError> {
        let outcome = operation.clone().apply(layout)?;
        let redo = Self::redo_record(layout, outcome.inverse());
        Ok(Self { operation, outcome, redo })
    }

    /// The outcome of the executed operation.
    pub fn outcome(&self) -> DockingOutcome {
        self.outcome.clone()
    }

    /// The executed operation.
    pub fn operation(&self) -> &DockingOperation {
        &self.operation
    }

    /// Undo the operation: replay the outcome's inverse data through the
    /// environment's own record-restore driver, restoring the
    /// pre-operation layout. The restored state is validated — the exact
    /// replay of a committed state is valid by construction, so an error
    /// here reports replay drift instead of propagating invalid state.
    pub fn undo(&self, layout: &mut DockLayout) -> Result<(), DockingError> {
        self.outcome.inverse().clone().replay_into(layout);
        layout.validate()?;
        Ok(())
    }

    /// Redo the operation: replay the forward record, restoring the
    /// post-operation layout (dock payloads re-placed, post-state
    /// collections restored, root re-designated, docks the operation
    /// removed deleted again). Validated like [`undo`](Self::undo).
    pub fn redo(&self, layout: &mut DockLayout) -> Result<(), DockingError> {
        self.redo.clone().replay_into(layout);
        layout.validate()?;
        Ok(())
    }

    /// The history entry naming the executed operation.
    pub fn history_entry(&self) -> HistoryEntry {
        let name = match &self.operation {
            DockingOperation::CreatePanel { .. } => "docking.create_panel",
            DockingOperation::CreateEmbedHost { .. } => "docking.create_embed_host",
            DockingOperation::InsertBeside { .. } => "docking.insert_beside",
            DockingOperation::TabJoin { .. } => "docking.tab_join",
            DockingOperation::ReorderTab { .. } => "docking.reorder_tab",
            DockingOperation::SetActiveTab { .. } => "docking.set_active_tab",
            DockingOperation::Remove { .. } => "docking.remove",
            DockingOperation::ResizeSiblings { .. } => "docking.resize_siblings",
            DockingOperation::SetDockCollapsed { .. } => "docking.set_dock_collapsed",
            DockingOperation::Embed { .. } => "docking.embed",
            DockingOperation::PromoteHost { .. } => "docking.promote_host",
            DockingOperation::ResizeRegion { .. } => "docking.resize_region",
            DockingOperation::SetFloatingRect { .. } => "docking.set_floating_rect",
        };
        HistoryEntry::new(name).undoable()
    }

    /// Build the forward replay record (the pre→post mirror).
    ///
    /// `UndoData::replay_into` is a generic state-restore driver — re-place
    /// dock payloads, restore collection snapshots, set the root, delete
    /// docks — applied to whatever record it is given. The inverse record
    /// is that driver fed the pre-operation state; the redo record is the
    /// same shape built forwards from the committed result:
    ///
    /// - `removed`: full **post** payloads of every dock the forward
    ///   replay must re-place — the docks the operation created plus docks
    ///   whose category swapped in place (promotions, demotions). This
    ///   covers the promoted-host case a collection snapshot cannot
    ///   express: the payload carries the whole `EmbedHost` role (regions,
    ///   floating collection, persistence) verbatim.
    /// - `containers`: **post** snapshots of every collection the
    ///   operation replaced (the same docks the inverse names — its
    ///   records carry the pre states). Deleted docks have no post state
    ///   and skip. An empty post region records `(None, None)`, which the
    ///   replay's clearing path resolves at the embedding baseline — the
    ///   environment's own cleared-region lifecycle.
    /// - `previous_root`: the post-operation root.
    /// - `created`: ids of the docks the operation removed, deleted again
    ///   by the forward replay.
    ///
    /// The record is order-commuting (every entry carries a final value),
    /// so it feeds `replay_into`'s load-bearing order unchanged.
    fn redo_record(post: &DockLayout, inverse: &UndoData) -> UndoData {
        let mut redo = UndoData::default();

        // Full post payloads for created and category-swapped docks.
        let mut replaced: Vec<DockId> = inverse.created.clone();
        for record in &inverse.containers {
            match record {
                FormerCollection::Promotion { id, .. } | FormerCollection::Demotion { id } => {
                    replaced.push(*id);
                }
                _ => {}
            }
        }
        for id in replaced {
            if let Some(dock) = post.get(id) {
                redo.removed.push(dock.clone());
            }
        }

        // Post collection snapshots for the docks the inverse names.
        for record in &inverse.containers {
            match record {
                FormerCollection::Split { id, .. } => {
                    if let Some(Dock { role: DockRole::Split { children, sizes, .. }, .. }) =
                        post.get(*id)
                    {
                        redo.containers.push(FormerCollection::Split {
                            id: *id,
                            children: children.clone(),
                            sizes: sizes.clone(),
                        });
                    }
                }
                FormerCollection::Tab { id, .. } => {
                    if let Some(
                        Dock { role: DockRole::TabGroup { members, active, .. }, .. },
                    ) = post.get(*id)
                    {
                        redo.containers.push(FormerCollection::Tab {
                            id: *id,
                            members: members.clone(),
                            active: *active,
                        });
                    }
                }
                FormerCollection::Region { host, edge, .. } => {
                    if let Some(Dock { role: DockRole::EmbedHost { regions, .. }, .. }) =
                        post.get(*host)
                    {
                        redo.containers.push(FormerCollection::Region {
                            host: *host,
                            edge: *edge,
                            previous: regions.on(*edge),
                            weight: regions.weight_on(*edge),
                        });
                    }
                }
                FormerCollection::Floating { host, .. } => {
                    if let Some(Dock { role: DockRole::EmbedHost { floating, .. }, .. }) =
                        post.get(*host)
                    {
                        redo.containers.push(FormerCollection::Floating {
                            host: *host,
                            entries: floating.clone(),
                        });
                    }
                }
                FormerCollection::Panel { id, .. } => {
                    if let Some(Dock { role: DockRole::Panel { collapsed, .. }, .. }) =
                        post.get(*id)
                    {
                        redo.containers.push(FormerCollection::Panel {
                            id: *id,
                            collapsed: *collapsed,
                        });
                    }
                }
                // Category swaps ride on the full payloads above.
                FormerCollection::Promotion { .. } | FormerCollection::Demotion { .. } => {}
            }
        }

        redo.previous_root = post.root();
        redo.created = inverse.removed.iter().map(|dock| dock.id).collect();
        redo
    }
}
