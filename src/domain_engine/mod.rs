//! # Domain Engine
//!
//! Application-specific domain logic composing Canvas2D + Timeline + Layers
//! + NodeGraph + Table + TextEditor + Viewport3D + Docking with App Engine
//! history, transactions, events, and signals.
//!
//! Real undo rides one **position-indexed executed-step registry**
//! (`ExecutedStep`) — every replayable step sits at the same vec index
//! as its history entry, entries recorded without a replay mechanism (the
//! plain-apply family, the docking restore) hold `None` gaps — so
//! interleaved environments replay in recorded order instead of relying
//! on per-environment vectors (the Phase-10 R-9-1 unification). Docking
//! persistence rides the established serialization conventions: the
//! model's own serde derive is the persisted form, and
//! [`DomainEngine::restore_docking`] validates the submitted model (the
//! Phase-3 gate) before adopting it.

pub mod operations;

use crate::app_engine::{
    EventBus, Event, HistoryEntry, HistoryStore, Signal, SignalBus,
    TransactionId, TransactionManager,
};
use crate::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, CanvasOperation, CanvasOutcome,
};
use crate::domain_foundation::environments::timeline::{
    Duration, Time, Timeline, TimelineError, TimelineOperation, TimelineOutcome,
};
use crate::domain_foundation::environments::layers::{
    Layers, LayersError, LayersOperation, LayersOutcome,
};
use crate::domain_foundation::environments::node_graph::{
    NodeGraph, NodeGraphError, NodeGraphOperation, NodeGraphOutcome,
};
use crate::domain_foundation::environments::table::{
    Table, TableError, TableOperation, TableOutcome,
};
use crate::domain_foundation::environments::text_editor::{
    Document, TextEditorError, TextEditorOperation, TextEditorOutcome,
};
use crate::domain_foundation::environments::viewport_3d::{
    Scene, Viewport3DError, Viewport3DOperation, Viewport3DOutcome,
};
use crate::domain_foundation::environments::docking::{
    DockLayout, DockingError, DockingOperation, DockingOutcome,
};
use operations::canvas_operations::ExecutedCanvasOperation;
use operations::docking_operations::ExecutedDockingOperation;
use std::sync::Arc;

/// One executed real-undo step, stored **position-aligned** with its
/// history entry (the Phase-10 R-9-1 unification: a single
/// `Vec<Option<_>>` registry indexed by history position instead of a
/// parallel vector per environment). Environments that record without a
/// replay mechanism (the plain-apply family) and non-replayable actions
/// (a restore) leave the `None` gap in the corresponding slot, so the
/// registry always mirrors the history vec 1:1.
#[derive(Debug)]
enum ExecutedStep {
    Canvas(ExecutedCanvasOperation),
    Docking(ExecutedDockingOperation),
}

impl ExecutedStep {
    fn undo(&self, canvas: &mut Canvas, docking: &mut DockLayout) -> Result<(), DomainError> {
        match self {
            ExecutedStep::Canvas(executed) => executed.undo(canvas).map_err(DomainError::from),
            ExecutedStep::Docking(executed) => executed.undo(docking).map_err(DomainError::from),
        }
    }

    fn redo(&self, canvas: &mut Canvas, docking: &mut DockLayout) -> Result<(), DomainError> {
        match self {
            ExecutedStep::Canvas(executed) => executed
                .redo(canvas)
                .map_err(DomainError::from)
                .map(|_| ()),
            ExecutedStep::Docking(executed) => executed.redo(docking).map_err(DomainError::from),
        }
    }
}

#[derive(Debug)]
pub enum DomainError {
    Canvas(CanvasError),
    Timeline(TimelineError),
    Layers(LayersError),
    NodeGraph(NodeGraphError),
    Table(TableError),
    TextEditor(TextEditorError),
    Viewport3D(Viewport3DError),
    Docking(DockingError),
}

impl From<CanvasError> for DomainError { fn from(e: CanvasError) -> Self { Self::Canvas(e) } }
impl From<TimelineError> for DomainError { fn from(e: TimelineError) -> Self { Self::Timeline(e) } }
impl From<LayersError> for DomainError { fn from(e: LayersError) -> Self { Self::Layers(e) } }
impl From<NodeGraphError> for DomainError { fn from(e: NodeGraphError) -> Self { Self::NodeGraph(e) } }
impl From<TableError> for DomainError { fn from(e: TableError) -> Self { Self::Table(e) } }
impl From<TextEditorError> for DomainError { fn from(e: TextEditorError) -> Self { Self::TextEditor(e) } }
impl From<Viewport3DError> for DomainError { fn from(e: Viewport3DError) -> Self { Self::Viewport3D(e) } }
impl From<DockingError> for DomainError { fn from(e: DockingError) -> Self { Self::Docking(e) } }

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::Canvas(e) => write!(f, "canvas: {e}"),
            DomainError::Timeline(e) => write!(f, "timeline: {e}"),
            DomainError::Layers(e) => write!(f, "layers: {e}"),
            DomainError::NodeGraph(e) => write!(f, "node_graph: {e}"),
            DomainError::Table(e) => write!(f, "table: {e}"),
            DomainError::TextEditor(e) => write!(f, "text_editor: {e}"),
            DomainError::Viewport3D(e) => write!(f, "viewport_3d: {e}"),
            DomainError::Docking(e) => write!(f, "docking: {e}"),
        }
    }
}
impl std::error::Error for DomainError {}

#[derive(Debug)]
pub struct DomainEngine {
    canvas: Canvas,
    timeline: Timeline,
    layers: Layers,
    node_graph: NodeGraph,
    table: Table,
    document: Document,
    scene: Scene,
    docking: DockLayout,
    history: HistoryStore,
    transaction_start_position: usize,
    transaction_manager: TransactionManager,
    event_bus: EventBus,
    signal_bus: Option<Arc<SignalBus>>,
    active_transaction: Option<TransactionId>,
    executed_steps: Vec<Option<ExecutedStep>>,
    saved_position: usize,
    max_history: usize,
    compound_depth: usize,
    events: Vec<Event>,
}

impl DomainEngine {
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
            timeline: Timeline::new(),
            layers: Layers::new(),
            node_graph: NodeGraph::new(),
            table: Table::new(),
            document: Document::new(),
            scene: Scene::new(),
            docking: DockLayout::new(),
            history: HistoryStore::new(),
            transaction_manager: TransactionManager::new(),
            event_bus: EventBus::new(),
            signal_bus: None,
            active_transaction: None,
            transaction_start_position: 0,
            executed_steps: Vec::new(),
            saved_position: 0,
            max_history: 0,
            compound_depth: 0,
            events: Vec::new(),
        }
    }

    pub fn with_signal_bus(mut self, signal_bus: Arc<SignalBus>) -> Self {
        self.signal_bus = Some(signal_bus);
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    // --- Environment access ---

    pub fn canvas(&self) -> &Canvas { &self.canvas }
    pub fn timeline(&self) -> &Timeline { &self.timeline }
    pub fn layers(&self) -> &Layers { &self.layers }
    pub fn node_graph(&self) -> &NodeGraph { &self.node_graph }
    pub fn table(&self) -> &Table { &self.table }
    pub fn document(&self) -> &Document { &self.document }
    pub fn scene(&self) -> &Scene { &self.scene }
    pub fn docking(&self) -> &DockLayout { &self.docking }
    pub fn history(&self) -> &HistoryStore { &self.history }
    pub fn event_bus(&self) -> &EventBus { &self.event_bus }
    pub fn event_bus_mut(&mut self) -> &mut EventBus { &mut self.event_bus }
    pub fn events(&self) -> &[Event] { &self.events }
    pub fn transaction_manager(&self) -> &TransactionManager { &self.transaction_manager }

    // --- D2: Dirty/clean tracking ---

    pub fn mark_saved(&mut self) {
        self.saved_position = self.history.position();
    }

    pub fn is_dirty(&self) -> bool {
        self.saved_position != self.history.position()
    }

    // --- D1: Event + Signal publication ---

    fn publish(&mut self, event_name: &str, signal_name: &str) {
        let event = Event::new(event_name).with_source("domain_engine");
        self.event_bus.publish(&event);
        self.events.push(event);
        if let Some(ref sb) = self.signal_bus {
            sb.emit(&Signal::new(signal_name).with_source("domain_engine"));
        }
    }

    // --- D3: Auto-rollback on error ---

    fn check_auto_rollback(&mut self, _error: &DomainError) {
        if self.active_transaction.is_some() {
            let _ = self.rollback_transaction();
        }
    }

    // --- D4: History limit ---

    fn enforce_history_limit(&mut self) {
        if self.max_history > 0 {
            let excess = self.history.entry_count().saturating_sub(self.max_history);
            if excess > 0 {
                // HistoryStore doesn't have truncate_old; clear all if over limit.
                // A proper implementation would add a truncate method to HistoryStore.
            }
        }
    }

    // --- Executed-step registry (Phase 10, R-9-1) ---

    /// Place an executed step at its **history position**.
    ///
    /// Single position-indexed registry for all real-undo environments:
    /// `executed_steps[i]` is `Some(step)` exactly when history entry `i`
    /// carries a replay step, and the `None` gap otherwise (plain-apply
    /// environments, a restore). `undo` / `redo` / transaction rollback
    /// walk history positions once and dispatch on the step, so
    /// interleaved environments stay aligned by construction instead of
    /// relying on per-environment vectors (the Phase-9 R-9-1 follow-up).
    fn push_executed_step(&mut self, pos: usize, step: ExecutedStep) {
        while self.executed_steps.len() < pos {
            self.executed_steps.push(None);
        }
        self.executed_steps.push(Some(step));
    }

    // --- Canvas operations ---

    pub fn execute_canvas(
        &mut self,
        operation: CanvasOperation,
    ) -> Result<CanvasOutcome, DomainError> {
        let pos = self.history.position();
        self.executed_steps.truncate(pos);

        let executed = match ExecutedCanvasOperation::execute(&mut self.canvas, operation) {
            Ok(e) => e,
            Err(e) => {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                return Err(err);
            }
        };
        let outcome = executed.outcome();

        let mut entry = executed.history_entry();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);
        self.push_executed_step(pos, ExecutedStep::Canvas(executed));

        self.enforce_history_limit();
        self.publish(
            event_name_for_canvas_outcome(&outcome),
            signal_name_for_canvas_outcome(&outcome),
        );

        Ok(outcome)
    }

    /// Backward-compatible alias.
    pub fn execute(
        &mut self,
        operation: CanvasOperation,
    ) -> Result<CanvasOutcome, CanvasError> {
        self.execute_canvas(operation).map_err(|e| match e {
            DomainError::Canvas(ce) => ce,
            _ => unreachable!(),
        })
    }

    // --- D5: Docking operations ---

    pub fn execute_docking(
        &mut self,
        operation: DockingOperation,
    ) -> Result<DockingOutcome, DomainError> {
        let pos = self.history.position();
        self.executed_steps.truncate(pos);

        let executed = match ExecutedDockingOperation::execute(&mut self.docking, operation) {
            Ok(e) => e,
            Err(e) => {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                return Err(err);
            }
        };
        let outcome = executed.outcome();

        let mut entry = executed.history_entry();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);
        self.push_executed_step(pos, ExecutedStep::Docking(executed));

        self.enforce_history_limit();
        self.publish(
            docking_outcome_event(&outcome),
            docking_outcome_signal(&outcome),
        );

        Ok(outcome)
    }

    // --- D5b: Docking persistence (Phase 10) ---

    /// Restore a persisted docking layout (PRD §12.3).
    ///
    /// The persisted form is the environment's own serde derive — the
    /// established serialization convention (every sibling environment
    /// ships derive-only serde; the host application serializes the
    /// cloned [`DockLayout`] and parses the payload back). The submitted
    /// model is validated **before** it is trusted: the Phase-3 gate
    /// (`DockLayout::validate`) *is* the imported-state report — first
    /// error, deterministic scan (root designation → reference validity
    /// + category parity, including the region-weight and floating-rect
    /// steady-state rules → placement multiplicity → acyclicity).
    ///
    /// **Rejection, never repair.** An invalid layout is declined
    /// wholesale (`DomainError::Docking`) and the engine's current
    /// layout, history, and events are untouched; v1 defines no missing-
    /// or outdated-reference recovery policy (PRD §12.3 SHOULD: prefer a
    /// meaningful error over silent, undocumented repair). The
    /// compatibility strategy is the form's own restore-compat design:
    /// `serde(default)` fields restore absent optional values (region
    /// weights at the baseline, persistent hosts, the collapse flag,
    /// floating rects), unknown payload fields are tolerated by serde,
    /// and anything structurally invalid is rejected — there is no
    /// version envelope or migration in v1.
    ///
    /// A valid layout is adopted **verbatim** — dock ids and the id
    /// counter ride inside the form (the Phase-1 §7.2 resolution:
    /// shipped, not re-issued), so a restored layout is collision-free
    /// by construction (DM-02/DM-04) and continued mutation keeps
    /// issuing fresh ids. Panel content is an opaque application-owned
    /// `ContentRef` — the host application remains responsible for
    /// restoring actual content (PRD §12.4). The adopt is recorded as a
    /// **non-undoable** `"docking.restored"` history entry (a trusted
    /// import, not a reversible operation — the restored file is the
    /// anchor), the engine is marked saved, and the adopt is published
    /// as `"docking.restored"` (event + signal).
    ///
    /// Registry note (R-9-1): the restore's history slot holds `None` —
    /// it is not a replayable step. History recorded *before* a restore
    /// describes the pre-restore session; post-restore docking
    /// operations replay against the restored layout, and an undo whose
    /// replay would drift out of validity surfaces as
    /// `DomainError::Docking` from the replay's validate guard.
    pub fn restore_docking(&mut self, layout: DockLayout) -> Result<(), DomainError> {
        layout.validate().map_err(DomainError::from)?;
        self.docking = layout;

        let entry = HistoryEntry::new("docking.restored").non_undoable();
        self.history.append(entry);
        self.mark_saved();
        self.publish("docking.restored", "docking.restored");
        Ok(())
    }

    // --- D6: Layers operations ---

    pub fn execute_layers(
        &mut self,
        operation: LayersOperation,
    ) -> Result<LayersOutcome, DomainError> {
        let outcome = operation.clone().apply(&mut self.layers)
            .map_err(|e| {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                err
            })?;

        let name = layers_op_name(&operation);
        let mut entry = HistoryEntry::new(name).undoable();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);

        self.publish(
            layers_outcome_event(&outcome),
            layers_outcome_signal(&outcome),
        );
        Ok(outcome)
    }

    // --- D6: NodeGraph operations ---

    pub fn execute_node_graph(
        &mut self,
        operation: NodeGraphOperation,
    ) -> Result<NodeGraphOutcome, DomainError> {
        let outcome = operation.clone().apply(&mut self.node_graph)
            .map_err(|e| {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                err
            })?;

        let name = node_graph_op_name(&operation);
        let mut entry = HistoryEntry::new(name).undoable();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);

        self.publish(
            node_graph_outcome_event(&outcome),
            node_graph_outcome_signal(&outcome),
        );
        Ok(outcome)
    }

    // --- D7: Table operations ---

    pub fn execute_table(
        &mut self,
        operation: TableOperation,
    ) -> Result<TableOutcome, DomainError> {
        let outcome = operation.clone().apply(&mut self.table)
            .map_err(|e| {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                err
            })?;

        let name = table_op_name(&operation);
        let mut entry = HistoryEntry::new(name).undoable();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);

        self.publish(
            table_outcome_event(&outcome),
            table_outcome_signal(&outcome),
        );
        Ok(outcome)
    }

    // --- D8: TextEditor operations ---

    pub fn execute_text_editor(
        &mut self,
        operation: TextEditorOperation,
    ) -> Result<TextEditorOutcome, DomainError> {
        let outcome = operation.clone().apply(&mut self.document)
            .map_err(|e| {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                err
            })?;

        let name = text_editor_op_name(&operation);
        let mut entry = HistoryEntry::new(name).undoable();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);

        self.publish(
            text_editor_outcome_event(&outcome),
            text_editor_outcome_signal(&outcome),
        );
        Ok(outcome)
    }

    // --- D9: Viewport3D operations ---

    pub fn execute_viewport_3d(
        &mut self,
        operation: Viewport3DOperation,
    ) -> Result<Viewport3DOutcome, DomainError> {
        let outcome = operation.clone().apply(&mut self.scene)
            .map_err(|e| {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                err
            })?;

        let name = viewport_3d_op_name(&operation);
        let mut entry = HistoryEntry::new(name).undoable();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);

        self.publish(
            viewport_3d_outcome_event(&outcome),
            viewport_3d_outcome_signal(&outcome),
        );
        Ok(outcome)
    }

    // --- Timeline operations ---

    pub fn execute_timeline(
        &mut self,
        operation: TimelineOperation,
    ) -> Result<TimelineOutcome, DomainError> {
        let outcome = operation.clone().apply(&mut self.timeline)
            .map_err(|e| {
                let err: DomainError = e.into();
                self.check_auto_rollback(&err);
                err
            })?;

        let name = timeline_op_name(&operation);
        let mut entry = HistoryEntry::new(name).undoable();
        if let Some(tx_id) = self.active_transaction {
            entry = entry.with_transaction(tx_id);
        }
        self.history.append(entry);

        self.publish(
            timeline_outcome_event(&outcome),
            timeline_outcome_signal(&outcome),
        );
        Ok(outcome)
    }

    // --- Transactions ---

    pub fn begin_transaction(&mut self) -> TransactionId {
        let tx_id = self.transaction_manager.begin();
        self.active_transaction = Some(tx_id);
        self.transaction_start_position = self.history.position();
        tx_id
    }

    pub fn commit_transaction(&mut self) -> Option<TransactionId> {
        self.active_transaction = None;
        self.transaction_manager.commit()
    }

    pub fn rollback_transaction(&mut self) -> Option<TransactionId> {
        let tx_id = self.active_transaction;
        self.active_transaction = None;
        let result = self.transaction_manager.rollback();
        if let Some(tx_id) = tx_id {
            // Undo the real-undo environments' operations from the
            // transaction start to now — dispatch on the one
            // position-indexed registry.
            let current_pos = self.history.position();
            for i in (self.transaction_start_position..current_pos).rev() {
                if let Some(Some(step)) = self.executed_steps.get(i) {
                    let _ = step.undo(&mut self.canvas, &mut self.docking);
                }
            }
            // Remove transaction entries from history.
            self.history.remove_transaction(tx_id);
            // Truncate the executed list to the new position.
            let new_pos = self.history.position();
            self.executed_steps.truncate(new_pos);
        }
        result
    }

    pub fn in_transaction(&self) -> bool {
        self.active_transaction.is_some()
    }

    // --- D10: Compound operations ---

    pub fn begin_compound(&mut self) -> TransactionId {
        self.compound_depth += 1;
        if self.compound_depth == 1 {
            let tx_id = self.begin_transaction();
            self.transaction_start_position = self.history.position();
            tx_id
        } else {
            self.active_transaction.unwrap()
        }
    }

    pub fn commit_compound(&mut self) -> Option<TransactionId> {
        if self.compound_depth > 0 {
            self.compound_depth -= 1;
            if self.compound_depth == 0 {
                self.commit_transaction()
            } else {
                self.active_transaction
            }
        } else {
            None
        }
    }

    pub fn rollback_compound(&mut self) -> Option<TransactionId> {
        if self.compound_depth > 0 {
            self.compound_depth = 0;
            self.rollback_transaction()
        } else {
            None
        }
    }

    // --- Undo / Redo ---

    pub fn undo(&mut self) -> Result<usize, DomainError> {
        let pos_before = self.history.position();
        let result = self.history.undo();
        let count = result.count();
        let pos_after = self.history.position();

        for i in (pos_after..pos_before).rev() {
            if let Some(Some(step)) = self.executed_steps.get(i) {
                step.undo(&mut self.canvas, &mut self.docking)?;
            }
        }

        self.publish("domain.undo", "domain.undo");
        Ok(count)
    }

    pub fn redo(&mut self) -> Result<usize, DomainError> {
        let pos_before = self.history.position();
        let result = self.history.redo();
        let count = result.count();
        let pos_after = self.history.position();

        for i in pos_before..pos_after {
            if let Some(Some(step)) = self.executed_steps.get(i) {
                step.redo(&mut self.canvas, &mut self.docking)?;
            }
        }

        self.publish("domain.redo", "domain.redo");
        Ok(count)
    }

    pub fn can_undo(&self) -> bool { self.history.can_undo() }
    pub fn can_redo(&self) -> bool { self.history.can_redo() }
}

impl Default for DomainEngine {
    fn default() -> Self { Self::new() }
}

// --- Name helpers ---

fn event_name_for_canvas_outcome(outcome: &CanvasOutcome) -> &'static str {
    match outcome {
        CanvasOutcome::Created { .. } => "canvas.object.created",
        CanvasOutcome::Moved { .. } => "canvas.object.moved",
        CanvasOutcome::Transformed { .. } => "canvas.object.transformed",
        CanvasOutcome::Removed { .. } => "canvas.object.removed",
        CanvasOutcome::MovedToFront { .. } => "canvas.object.moved_to_front",
        CanvasOutcome::MovedToBack { .. } => "canvas.object.moved_to_back",
        CanvasOutcome::MovedForward { .. } => "canvas.object.moved_forward",
        CanvasOutcome::MovedBackward { .. } => "canvas.object.moved_backward",
        CanvasOutcome::Reordered { .. } => "canvas.object.reordered",
        CanvasOutcome::Duplicated { .. } => "canvas.object.duplicated",
        CanvasOutcome::Cleared { .. } => "canvas.cleared",
    }
}

fn signal_name_for_canvas_outcome(outcome: &CanvasOutcome) -> &'static str {
    match outcome {
        CanvasOutcome::Created { .. } => "canvas.object.created",
        CanvasOutcome::Moved { .. } => "canvas.object.moved",
        CanvasOutcome::Transformed { .. } => "canvas.object.transformed",
        CanvasOutcome::Removed { .. } => "canvas.object.removed",
        _ => "canvas.changed",
    }
}

fn docking_outcome_event(outcome: &DockingOutcome) -> &'static str {
    match outcome {
        DockingOutcome::Created { .. } => "docking.dock.created",
        DockingOutcome::InsertedBeside { .. } => "docking.dock.inserted_beside",
        DockingOutcome::TabJoined { .. } => "docking.tab.joined",
        DockingOutcome::TabReordered { .. } => "docking.tab.reordered",
        DockingOutcome::ActiveTabSelected { .. } => "docking.tab.active_selected",
        DockingOutcome::Removed { .. } => "docking.dock.removed",
        DockingOutcome::SiblingsResized { .. } => "docking.split.resized",
        DockingOutcome::DockCollapsedSet { .. } => "docking.panel.collapsed_set",
        DockingOutcome::Embedded { .. } => "docking.dock.embedded",
        DockingOutcome::HostPromoted { .. } => "docking.host.promoted",
        DockingOutcome::RegionResized { .. } => "docking.region.resized",
        DockingOutcome::FloatingRectSet { .. } => "docking.floating.rect_set",
    }
}

fn docking_outcome_signal(outcome: &DockingOutcome) -> &'static str {
    match outcome {
        DockingOutcome::Created { .. } => "docking.dock.created",
        DockingOutcome::Removed { .. } => "docking.dock.removed",
        DockingOutcome::SiblingsResized { .. } => "docking.split.resized",
        DockingOutcome::RegionResized { .. } => "docking.region.resized",
        DockingOutcome::FloatingRectSet { .. } => "docking.floating.rect_set",
        _ => "docking.changed",
    }
}

fn layers_op_name(op: &LayersOperation) -> &'static str {
    match op {
        LayersOperation::CreateLayer { .. } => "layers.create_layer",
        LayersOperation::CreateChild { .. } => "layers.create_child",
        LayersOperation::RemoveLayer { .. } => "layers.remove_layer",
        LayersOperation::RenameLayer { .. } => "layers.rename_layer",
        LayersOperation::SetVisibility { .. } => "layers.set_visibility",
        LayersOperation::SetLocked { .. } => "layers.set_locked",
        LayersOperation::ReorderSibling { .. } => "layers.reorder_sibling",
        LayersOperation::Reparent { .. } => "layers.reparent",
        LayersOperation::Clear => "layers.clear",
        LayersOperation::DuplicateLayer { .. } => "layers.duplicate_layer",
    }
}

fn layers_outcome_event(outcome: &LayersOutcome) -> &'static str {
    match outcome {
        LayersOutcome::LayerCreated { .. } => "layers.layer.created",
        LayersOutcome::LayerRemoved { .. } => "layers.layer.removed",
        LayersOutcome::LayerRenamed { .. } => "layers.layer.renamed",
        LayersOutcome::VisibilityChanged { .. } => "layers.visibility.changed",
        LayersOutcome::LockedChanged { .. } => "layers.locked.changed",
        LayersOutcome::SiblingReordered { .. } => "layers.sibling.reordered",
        LayersOutcome::Reparented { .. } => "layers.reparented",
        LayersOutcome::Cleared => "layers.cleared",
        LayersOutcome::LayerDuplicated { .. } => "layers.layer.duplicated",
    }
}

fn layers_outcome_signal(outcome: &LayersOutcome) -> &'static str {
    match outcome {
        LayersOutcome::LayerCreated { .. } => "layers.layer.created",
        LayersOutcome::LayerRemoved { .. } => "layers.layer.removed",
        LayersOutcome::VisibilityChanged { .. } => "layers.visibility.changed",
        // In the match, add before the wildcard:
        LayersOutcome::LayerDuplicated { .. } => "layers.layer.duplicated",
        _ => "layers.changed",
    }
}

fn node_graph_op_name(op: &NodeGraphOperation) -> &'static str {
    match op {
        NodeGraphOperation::CreateNode { .. } => "node_graph.create_node",
        NodeGraphOperation::RemoveNode { .. } => "node_graph.remove_node",
        NodeGraphOperation::RenameNode { .. } => "node_graph.rename_node",
        NodeGraphOperation::MoveNode { .. } => "node_graph.move_node",
        NodeGraphOperation::SetNodeTransform { .. } => "node_graph.set_transform",
        NodeGraphOperation::AddInputPort { .. } => "node_graph.add_input_port",
        NodeGraphOperation::AddOutputPort { .. } => "node_graph.add_output_port",
        NodeGraphOperation::RemovePort { .. } => "node_graph.remove_port",
        NodeGraphOperation::Connect { .. } => "node_graph.connect",
        NodeGraphOperation::Disconnect { .. } => "node_graph.disconnect",
        NodeGraphOperation::RenamePort { .. } => "node_graph.rename_port",
        NodeGraphOperation::DuplicateNode { .. } => "node_graph.duplicate_node",
        NodeGraphOperation::Clear => "node_graph.clear",
    }
}

fn node_graph_outcome_event(outcome: &NodeGraphOutcome) -> &'static str {
    match outcome {
        NodeGraphOutcome::NodeCreated { .. } => "node_graph.node.created",
        NodeGraphOutcome::NodeRemoved(_) => "node_graph.node.removed",
        NodeGraphOutcome::NodeRenamed { .. } => "node_graph.node.renamed",
        NodeGraphOutcome::NodeMoved { .. } => "node_graph.node.moved",
        NodeGraphOutcome::NodeTransformSet { .. } => "node_graph.node.transform_set",
        NodeGraphOutcome::PortAdded { .. } => "node_graph.port.added",
        NodeGraphOutcome::PortRemoved(_) => "node_graph.port.removed",
        NodeGraphOutcome::Connected { .. } => "node_graph.connected",
        NodeGraphOutcome::Disconnected { .. } => "node_graph.disconnected",
        NodeGraphOutcome::PortRenamed { .. } => "node_graph.port.renamed",
        NodeGraphOutcome::NodeDuplicated { .. } => "node_graph.node.duplicated",
        NodeGraphOutcome::Cleared { .. } => "node_graph.cleared",
    }
}

fn node_graph_outcome_signal(outcome: &NodeGraphOutcome) -> &'static str {
    match outcome {
        NodeGraphOutcome::NodeCreated { .. } => "node_graph.node.created",
        NodeGraphOutcome::NodeRemoved(_) => "node_graph.node.removed",
        NodeGraphOutcome::Connected { .. } => "node_graph.connected",
        NodeGraphOutcome::Disconnected { .. } => "node_graph.disconnected",
        _ => "node_graph.changed",
    }
}

fn table_op_name(op: &TableOperation) -> &'static str {
    match op {
        TableOperation::InsertRow { .. } => "table.insert_row",
        TableOperation::InsertColumn { .. } => "table.insert_column",
        TableOperation::RemoveRow { .. } => "table.remove_row",
        TableOperation::RemoveColumn { .. } => "table.remove_column",
        TableOperation::SetCellValue { .. } => "table.set_cell_value",
        TableOperation::ClearCell { .. } => "table.clear_cell",
        TableOperation::MoveRow { .. } => "table.move_row",
        TableOperation::MoveColumn { .. } => "table.move_column",
        TableOperation::RenameRow { .. } => "table.rename_row",
        TableOperation::RenameColumn { .. } => "table.rename_column",
        TableOperation::MergeCells { .. } => "table.merge_cells",
        TableOperation::UnmergeCells { .. } => "table.unmerge_cells",
        TableOperation::ClearTable => "table.clear",
    }
}

fn table_outcome_event(outcome: &TableOutcome) -> &'static str {
    match outcome {
        TableOutcome::RowInserted { .. } => "table.row.inserted",
        TableOutcome::ColumnInserted { .. } => "table.column.inserted",
        TableOutcome::RowRemoved { .. } => "table.row.removed",
        TableOutcome::ColumnRemoved { .. } => "table.column.removed",
        TableOutcome::CellValueSet { .. } => "table.cell.value_set",
        TableOutcome::CellCleared { .. } => "table.cell.cleared",
        TableOutcome::RowMoved { .. } => "table.row.moved",
        TableOutcome::ColumnMoved { .. } => "table.column.moved",
        TableOutcome::RowRenamed { .. } => "table.row.renamed",
        TableOutcome::ColumnRenamed { .. } => "table.column.renamed",
        TableOutcome::CellsMerged => "table.cells.merged",
        TableOutcome::CellsUnmerged => "table.cells.unmerged",
        TableOutcome::TableCleared => "table.cleared",
    }
}

fn table_outcome_signal(outcome: &TableOutcome) -> &'static str {
    match outcome {
        TableOutcome::RowInserted { .. } => "table.row.inserted",
        TableOutcome::RowRemoved { .. } => "table.row.removed",
        TableOutcome::CellValueSet { .. } => "table.cell.value_set",
        _ => "table.changed",
    }
}

fn text_editor_op_name(op: &TextEditorOperation) -> &'static str {
    match op {
        TextEditorOperation::Insert { .. } => "text_editor.insert",
        TextEditorOperation::Delete { .. } => "text_editor.delete",
        TextEditorOperation::Replace { .. } => "text_editor.replace",
        TextEditorOperation::Clear => "text_editor.clear",
        TextEditorOperation::Append { .. } => "text_editor.append",
        TextEditorOperation::DeleteLine { .. } => "text_editor.delete_line",
        TextEditorOperation::ReplaceLine { .. } => "text_editor.replace_line",
    }
}

fn text_editor_outcome_event(outcome: &TextEditorOutcome) -> &'static str {
    match outcome {
        TextEditorOutcome::Inserted { .. } => "text_editor.inserted",
        TextEditorOutcome::Deleted { .. } => "text_editor.deleted",
        TextEditorOutcome::Replaced { .. } => "text_editor.replaced",
        TextEditorOutcome::Cleared { .. } => "text_editor.cleared",
        TextEditorOutcome::Appended { .. } => "text_editor.appended",
        TextEditorOutcome::LineDeleted { .. } => "text_editor.line_deleted",
        TextEditorOutcome::LineReplaced { .. } => "text_editor.line_replaced",
    }
}

fn text_editor_outcome_signal(outcome: &TextEditorOutcome) -> &'static str {
    match outcome {
        TextEditorOutcome::Inserted { .. } => "text_editor.inserted",
        TextEditorOutcome::Deleted { .. } => "text_editor.deleted",
        _ => "text_editor.changed",
    }
}

fn viewport_3d_op_name(op: &Viewport3DOperation) -> &'static str {
    match op {
        Viewport3DOperation::Create { .. } => "viewport_3d.create",
        Viewport3DOperation::Move { .. } => "viewport_3d.move",
        Viewport3DOperation::Rotate { .. } => "viewport_3d.rotate",
        Viewport3DOperation::Scale { .. } => "viewport_3d.scale",
        Viewport3DOperation::Transform { .. } => "viewport_3d.transform",
        Viewport3DOperation::Remove { .. } => "viewport_3d.remove",
        Viewport3DOperation::SetParent { .. } => "viewport_3d.set_parent",
        Viewport3DOperation::Clear => "viewport_3d.clear",
    }
}

fn viewport_3d_outcome_event(outcome: &Viewport3DOutcome) -> &'static str {
    match outcome {
        Viewport3DOutcome::Created { .. } => "viewport_3d.object.created",
        Viewport3DOutcome::Moved { .. } => "viewport_3d.object.moved",
        Viewport3DOutcome::Rotated { .. } => "viewport_3d.object.rotated",
        Viewport3DOutcome::Scaled { .. } => "viewport_3d.object.scaled",
        Viewport3DOutcome::Transformed { .. } => "viewport_3d.object.transformed",
        Viewport3DOutcome::Removed { .. } => "viewport_3d.object.removed",
        Viewport3DOutcome::ParentSet { .. } => "viewport_3d.object.parent_set",
        Viewport3DOutcome::Cleared { .. } => "viewport_3d.cleared",
    }
}

fn viewport_3d_outcome_signal(outcome: &Viewport3DOutcome) -> &'static str {
    match outcome {
        Viewport3DOutcome::Created { .. } => "viewport_3d.object.created",
        Viewport3DOutcome::Removed { .. } => "viewport_3d.object.removed",
        _ => "viewport_3d.changed",
    }
}

fn timeline_op_name(op: &TimelineOperation) -> &'static str {
    match op {
        TimelineOperation::CreateTrack => "timeline.create_track",
        TimelineOperation::RemoveTrack { .. } => "timeline.remove_track",
        TimelineOperation::InsertClip { .. } => "timeline.insert_clip",
        TimelineOperation::RemoveClip { .. } => "timeline.remove_clip",
        TimelineOperation::MoveClip { .. } => "timeline.move_clip",
        TimelineOperation::AddKeyframe { .. } => "timeline.add_keyframe",
        TimelineOperation::RemoveKeyframe { .. } => "timeline.remove_keyframe",
        TimelineOperation::TrimClip { .. } => "timeline.trim_clip",
        TimelineOperation::SplitClip { .. } => "timeline.split_clip",
        TimelineOperation::ClearTimeline => "timeline.clear",
        TimelineOperation::SetKeyframeValue { .. } => "timeline.set_keyframe_value",
    }
}

fn timeline_outcome_event(outcome: &TimelineOutcome) -> &'static str {
    match outcome {
        TimelineOutcome::TrackCreated { .. } => "timeline.track.created",
        TimelineOutcome::TrackRemoved { .. } => "timeline.track.removed",
        TimelineOutcome::ClipInserted { .. } => "timeline.clip.inserted",
        TimelineOutcome::ClipRemoved { .. } => "timeline.clip.removed",
        TimelineOutcome::ClipMoved { .. } => "timeline.clip.moved",
        TimelineOutcome::KeyframeAdded { .. } => "timeline.keyframe.added",
        TimelineOutcome::KeyframeRemoved { .. } => "timeline.keyframe.removed",
        TimelineOutcome::ClipTrimmed { .. } => "timeline.clip.trimmed",
        TimelineOutcome::ClipSplit { .. } => "timeline.clip.split",
        TimelineOutcome::TimelineCleared => "timeline.cleared",
        TimelineOutcome::KeyframeValueChanged { .. } => "timeline.keyframe.value_changed",
    }
}

fn timeline_outcome_signal(outcome: &TimelineOutcome) -> &'static str {
    match outcome {
        TimelineOutcome::TrackCreated { .. } => "timeline.track.created",
        TimelineOutcome::TrackRemoved { .. } => "timeline.track.removed",
        TimelineOutcome::ClipInserted { .. } => "timeline.clip.inserted",
        TimelineOutcome::ClipRemoved { .. } => "timeline.clip.removed",
        _ => "timeline.changed",
    }
}