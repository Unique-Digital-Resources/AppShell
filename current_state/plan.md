# Current Feature Plan — Docking Environment (12 phases)

*This is `current_state/plan.md` — the plan of record for the currently tracked
feature task: establishing the headless **docking environment** in Domain
Foundation. The current phase pointer, established state, and reading list live in
`current_state/CHECKPOINT.md`; per-phase execution follows the protocol in
`agents/recipes/run_plan_phase.md`. Requirements and acceptance criteria:
`current_state/PRD.md`; conceptual design basis: `current_state/design/`.*

The plan below follows the proposed folder structure and Docking Environment PRD. It treats docking as a new headless Domain Foundation environment, integrated into the existing Domain Engine rather than as a UI component.

Each phase identifies its objective, implementation tasks, related files, and completion criteria. File names are proposed and should be aligned with the existing AppShell conventions before implementation.

## Phase overview

### 01

Architecture and integration review

Identify existing conventions and define the environment boundary.

### 02

Core domain model

Establish docking elements, hierarchy, slots, and identity.

### 03

Validation and structural invariants

Define what constitutes a valid docking layout.

### 04

Basic structural operations

Implement creation, splitting, moving, grouping, and removal.

### 05

Normalization and atomic mutations

Guarantee consistent structures after successful operations.

### 06

Sizing and resizing

Define relative allocation, constraints, and collapse behavior.

### 07

Embedding and host lifecycle

Support embedded hosts and host promotion or demotion.

### 08

Floating docking structures

Support independent floating roots and restoration to a host.

### 09

Domain Engine integration

Expose docking operations through AppShell's existing patterns.

### 10

History, persistence, and restoration

Integrate with established state, serialization, and undo/redo mechanisms.

### 11

Interaction contract and frontend handoff

Specify drag/drop intent and frontend responsibilities without implementing UI.

### 12

Verification and release readiness

Test invariants, operation sequences, failure handling, and acceptance criteria.

## Phase 1 — Architecture and integration review

Goal: Ensure the new environment fits AppShell's existing architecture instead of introducing a parallel design.

### Tasks

- Inspect the existing `canvas_2d`, `layers`, and `node_graph` environments.
- Identify how each environment defines models, operations, errors, and public exports.
- Inspect `domain_engine/operations` to understand command handlers and operation execution.
- Identify the established patterns for state changes, history, persistence, and serialization.
- Finalize the docking model's terminology and public boundaries.
- Decide how docking IDs and references will be represented using existing project conventions.

### Related files

- `domain_foundation/environments/canvas_2d/`
- `domain_foundation/environments/layers/`
- `domain_foundation/environments/node_graph/`
- `domain_engine/operations/`
- `domain_foundation/environments/mod.rs`
- `domain_foundation/mod.rs`

### Completion criteria

- The proposed docking model is documented.
- Integration points are identified from actual source code.
- No duplicate command, history, or persistence mechanism is planned.

## Phase 2 — Core domain model

Goal: Represent a docking layout without requiring any graphical interface.

### Tasks

- Define the central docking model and its designated root.
- Define stable identifiers and references for dockable elements.
- Define panel, container, and placement-slot concepts.
- Define split orientation and child ordering.
- Establish how tabs and host relationships fit into the model.
- Define the valid empty-layout representation.
- Ensure the model can represent nested layouts.

### Related files

- `model.rs`
- `panel.rs`
- `container.rs`
- `slot.rs`
- `mod.rs`

### Completion criteria

- A layout can be represented entirely as domain data.
- Nested structures can be inspected and traversed.
- Identity and ownership rules are unambiguous.
- No rendering coordinates or frontend widgets are required to define the structure.

## Phase 3 — Validation and structural invariants

Goal: Establish the rules every valid docking state must satisfy.

### Tasks

Define and test invariants such as:

- Every element has a valid identity.
- Every non-root element has a valid structural owner or explicitly defined host relationship.
- A structural element cannot be its own ancestor.
- Cycles are rejected.
- Elements cannot appear in incompatible locations simultaneously.
- Tab groups have valid membership and active-tab state.
- Split containers have valid child counts and sizing data.
- Root and host references are valid.
- Invalid operations cannot leave partially updated state.

### Related files

- `validation.rs`
- `error.rs`
- `tests/fixtures.rs`
- `tests/structure.rs`
- `tests/validation.rs`

### Completion criteria

- Valid and invalid model fixtures are covered by tests.
- Validation errors identify the violated rule.
- Validation can be used independently of the UI.

## Phase 4 — Basic structural operations

Goal: Implement the minimum useful docking behavior.

### Tasks

Implement domain operations for:

1. Creating dockable panels and initial layout structures.
2. Inserting a panel beside an existing panel using horizontal or vertical splits.
3. Moving a panel to another valid position.
4. Creating a tab group from eligible panels.
5. Moving a panel into an existing tab group.
6. Reordering tabs and selecting the active tab.
7. Removing a panel.
8. Replacing or updating a panel's structural placement through explicit operations.

Define the behavior of center, edge, and occupied-slot operations. In particular, an occupied edge should be rejected unless the operation explicitly defines how to handle that occupancy.

### Related files

- `operations.rs`
- `model.rs`
- `container.rs`
- `tab_group.rs`
- `slot.rs`
- `error.rs`
- `tests/splitting.rs`
- `tests/tab_groups.rs`

### Completion criteria

- Each operation has defined preconditions and postconditions.
- Moving an element preserves its identity.
- Invalid moves are rejected.
- Basic operations maintain the invariants defined in Phase 3.

## Phase 5 — Normalization and atomic mutations

Goal: Prevent layouts from accumulating unnecessary or invalid structure after changes.

### Tasks

- Define when empty containers should be removed.
- Define when a split container with one remaining child should be collapsed.
- Preserve the relative ordering of surviving children.
- Define how sizing information is redistributed after removal or movement.
- Establish the order of mutation, normalization, validation, and commit.
- Ensure a failed operation leaves the original state unchanged.

### Related files

- `normalization.rs`
- `operations.rs`
- `validation.rs`
- `error.rs`
- `tests/normalization.rs`

### Completion criteria

- Structural cleanup is deterministic.
- Normalization never silently discards valid panels or tab groups.
- Failed operations do not leave partial mutations.
- Repeated normalization does not keep changing an already normalized layout.

## Phase 6 — Sizing and resizing

Goal: Define how docking structures divide and allocate space.

### Tasks

- Define relative sizing weights for split children.
- Define minimum and maximum size constraints.
- Define how resizing updates sibling allocations.
- Define behavior when constraints conflict.
- Define collapse and expansion rules for eligible elements.
- Define how sizing is adjusted after insertion, removal, or movement.
- Separate logical size allocation from actual screen geometry.

### Related files

- `sizing.rs`
- `container.rs`
- `operations.rs`
- `validation.rs`
- `tests/sizing.rs`

### Completion criteria

- Resizing preserves valid allocation data.
- Constraints are handled predictably.
- Layout changes do not produce invalid sizing values.
- The model remains independent of pixels and frontend rendering.

## Phase 7 — Embedding and host lifecycle

Goal: Support docking structures hosted inside other dockable structures.

### Tasks

- Define host identity and the relationship between a host and its embedded docking structure.
- Define valid host slots and occupancy rules.
- Implement embedding and removal of embedded structures.
- Implement host promotion when a host's own content becomes part of a broader docking arrangement.
- Define host demotion or reconstruction when a promoted structure is separated again.
- Preserve host identity and content according to the PRD's lifecycle rules.
- Distinguish temporary hosting relationships from persistent ones where required.

### Related files

- `embed_host.rs`
- `slot.rs`
- `model.rs`
- `operations.rs`
- `normalization.rs`
- `validation.rs`
- `tests/embedding.rs`

### Completion criteria

- Host relationships are explicit and valid.
- Embedding does not create cycles or duplicate ownership.
- Promotion and demotion preserve the intended identities and content.
- Removing a host has a defined outcome for its embedded structures.

## Phase 8 — Floating docking structures

Goal: Allow complete panels or groups to exist outside the main docking root.

### Tasks

- Define floating-root representation.
- Support floating a panel, tab group, or eligible subtree.
- Preserve internal structure when a group or subtree is floated.
- Define how floating structures are attached to or detached from the main layout.
- Support docking a floating structure back into a valid target.
- Define how floating state interacts with embedding and host relationships.
- Keep window creation and OS-specific positioning outside the domain environment.

### Related files

- `floating.rs`
- `model.rs`
- `operations.rs`
- `slot.rs`
- `sizing.rs`
- `tests/floating.rs`

### Completion criteria

- Floating structures remain valid independently of the main root.
- Floating and re-docking preserve the intended internal structure.
- The domain model does not assume a particular window manager or UI toolkit.

## Phase 9 — Domain Engine integration

Goal: Make the environment usable through AppShell's existing operation execution patterns.

### Tasks

- Expose the environment through the appropriate module declarations.
- Add docking operation handling following the actual conventions of existing environments.
- Connect operation execution to the Docking Environment model.
- Map domain errors into the appropriate existing error or result pattern.
- Ensure callers can access docking operations without bypassing validation.
- Avoid duplicating state ownership between the Domain Engine and the environment.

### Related files

- `domain_engine/operations/docking_command_handler.rs`
- `domain_engine/operations/docking_operations.rs`
- `domain_engine/operations/mod.rs`
- `domain_engine/mod.rs`
- `domain_foundation/environments/mod.rs`
- `domain_foundation/environments/docking/mod.rs`

### Completion criteria

- Docking operations are accessible through the established AppShell integration.
- Invalid state changes are rejected consistently.
- Existing environments continue to work without regressions.

## Phase 10 — History, persistence, and restoration

Goal: Make docking state reliable across undo/redo and application restarts.

### Tasks

- Determine how the existing AppShell history system records environment changes.
- Integrate docking mutations with existing undo/redo and transaction mechanisms.
- Define serialization of docking models, IDs, relationships, and sizing state.
- Define how invalid or outdated persisted state is handled.
- Restore layouts while validating references and structural invariants.
- Define restoration behavior when a referenced panel or host is unavailable.
- Test compatibility with the project's existing serialization conventions.

### Related files

- Docking environment model and operations files.
- Existing `app_engine/history/`
- Existing `app_engine/persistence/`
- Existing `app_engine/serialization/`

### Completion criteria

- Docking changes can be undone and redone using the existing history system.
- A saved layout can be restored into an equivalent valid domain state.
- Missing or invalid references are handled according to explicit rules.
- No docking-specific history or persistence framework is introduced unnecessarily.

## Phase 11 — Interaction contract and frontend handoff

Goal: Define how a frontend can drive docking behavior without moving interaction logic into the headless environment.

### Tasks

Specify the contract for:

- Beginning and cancelling a drag operation.
- Identifying the dragged element or subtree.
- Representing a proposed drop target.
- Expressing top, bottom, left, right, and center drop intent.
- Distinguishing split, tab-group, and embedding operations.
- Determining whether a proposed operation is eligible.
- Committing the operation after the user confirms the drop.
- Reporting validation failures to the frontend.

The frontend remains responsible for hit-testing, pointer tracking, drop overlays, visual previews, animations, and window management.

### Related files

- `operations.rs`
- `slot.rs`
- `error.rs`
- `domain_engine/operations/docking_command_handler.rs`

A separate interaction or drag-session file should be added only if the domain API genuinely needs persistent interaction state. Purely visual drag state should remain in the frontend.

### Completion criteria

- The frontend can express docking intent using domain-level inputs.
- The environment can validate and execute those requests without knowing about mouse coordinates.
- Preview state is not accidentally persisted as committed layout state.

## Phase 12 — Verification and release readiness

Goal: Verify the environment against the PRD and ensure it is ready for integration by applications.

### Tasks

- Test each structural operation independently.
- Test operation sequences, such as split → move → tab group → remove.
- Test host promotion and demotion.
- Test floating and re-docking of nested structures.
- Test sizing constraints and normalization.
- Test invalid operations and rollback behavior.
- Test serialization and restoration.
- Test undo/redo through the existing AppShell integration.
- Add integration tests and documentation.
- Review the public API and remove unnecessary abstractions.

### Related files

- `tests/`
- `domain_foundation/testing/`
- `domain_engine/operations/`
- Documentation and examples for the new environment.

### Completion criteria

- All PRD acceptance criteria have corresponding tests.
- No structural invariants are violated by supported operations.
- Public operations have documented behavior and errors.
- The environment works without a frontend and is ready for a UI adapter.

## Recommended implementation order

The phases above are sequential where dependencies require it, but some work can overlap. The most important dependency chain is:

\#chatgpt-mermaid-\_r_2le\_{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";font-size:16px;fill:rgb(237, 237, 237);}@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}@keyframes dash{to{stroke-dashoffset:0;}}#chatgpt-mermaid-\_r_2le\_ .edge-animation-slow{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}#chatgpt-mermaid-\_r_2le\_ .edge-animation-fast{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}#chatgpt-mermaid-\_r_2le\_ .error-icon{fill:rgb(48, 48, 48);}#chatgpt-mermaid-\_r_2le\_ .error-text{fill:rgb(237, 237, 237);stroke:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ .edge-thickness-normal{stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .edge-thickness-thick{stroke-width:3.5px;}#chatgpt-mermaid-\_r_2le\_ .edge-pattern-solid{stroke-dasharray:0;}#chatgpt-mermaid-\_r_2le\_ .edge-thickness-invisible{stroke-width:0;fill:none;}#chatgpt-mermaid-\_r_2le\_ .edge-pattern-dashed{stroke-dasharray:3;}#chatgpt-mermaid-\_r_2le\_ .edge-pattern-dotted{stroke-dasharray:2;}#chatgpt-mermaid-\_r_2le\_ .marker{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2le\_ .marker.cross{stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2le\_ svg{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";font-size:16px;}#chatgpt-mermaid-\_r_2le\_ p{margin:0;}#chatgpt-mermaid-\_r_2le\_ .label{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ .cluster-label text{fill:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ .cluster-label span{color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ .cluster-label span p{background-color:transparent;}#chatgpt-mermaid-\_r_2le\_ .label text,#chatgpt-mermaid-\_r_2le\_ span{fill:rgb(237, 237, 237);color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ .node rect,#chatgpt-mermaid-\_r_2le\_ .node circle,#chatgpt-mermaid-\_r_2le\_ .node ellipse,#chatgpt-mermaid-\_r_2le\_ .node polygon,#chatgpt-mermaid-\_r_2le\_ .node path{fill:rgb(52, 24, 11);stroke:rgb(172, 79, 35);stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .rough-node .label text,#chatgpt-mermaid-\_r_2le\_ .node .label text,#chatgpt-mermaid-\_r_2le\_ .image-shape .label,#chatgpt-mermaid-\_r_2le\_ .icon-shape .label{text-anchor:middle;}#chatgpt-mermaid-\_r_2le\_ .node .katex path{fill:#000;stroke:#000;stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .rough-node .label,#chatgpt-mermaid-\_r_2le\_ .node .label,#chatgpt-mermaid-\_r_2le\_ .image-shape .label,#chatgpt-mermaid-\_r_2le\_ .icon-shape .label{text-align:center;}#chatgpt-mermaid-\_r_2le\_ .node.clickable{cursor:pointer;}#chatgpt-mermaid-\_r_2le\_ .root .anchor path{fill:rgb(175, 175, 175)!important;stroke-width:0;stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2le\_ .arrowheadPath{fill:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2le\_ .edgePath .path{stroke:rgb(175, 175, 175);stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .flowchart-link{stroke:rgb(175, 175, 175);fill:none;}#chatgpt-mermaid-\_r_2le\_ .edgeLabel{background-color:rgb(0, 0, 0);text-align:center;}#chatgpt-mermaid-\_r_2le\_ .edgeLabel p{background-color:rgb(0, 0, 0);}#chatgpt-mermaid-\_r_2le\_ .edgeLabel rect{opacity:0.5;background-color:rgb(0, 0, 0);fill:rgb(0, 0, 0);}#chatgpt-mermaid-\_r_2le\_ .labelBkg{background-color:rgba(0, 0, 0, 0.5);}#chatgpt-mermaid-\_r_2le\_ .cluster rect{fill:rgb(48, 48, 48);stroke:rgba(255, 255, 255, 0.15);stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .cluster text{fill:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ .cluster span{color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ div.mermaidTooltip{position:absolute;text-align:center;max-width:200px;padding:2px;font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";font-size:12px;background:rgb(48, 48, 48);border:1px solid rgba(255, 255, 255, 0.15);border-radius:2px;pointer-events:none;z-index:100;}#chatgpt-mermaid-\_r_2le\_ .flowchartTitleText{text-anchor:middle;font-size:18px;fill:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2le\_ rect.text{fill:none;stroke-width:0;}#chatgpt-mermaid-\_r_2le\_ .icon-shape,#chatgpt-mermaid-\_r_2le\_ .image-shape{background-color:rgb(0, 0, 0);text-align:center;}#chatgpt-mermaid-\_r_2le\_ .icon-shape p,#chatgpt-mermaid-\_r_2le\_ .image-shape p{background-color:rgb(0, 0, 0);padding:2px;}#chatgpt-mermaid-\_r_2le\_ .icon-shape .label rect,#chatgpt-mermaid-\_r_2le\_ .image-shape .label rect{opacity:0.5;background-color:rgb(0, 0, 0);fill:rgb(0, 0, 0);}#chatgpt-mermaid-\_r_2le\_ .label-icon{display:inline-block;height:1em;overflow:visible;vertical-align:-0.125em;}#chatgpt-mermaid-\_r_2le\_ .node .label-icon path{fill:currentColor;stroke:revert;stroke-width:revert;}#chatgpt-mermaid-\_r_2le\_ .node .neo-node{stroke:rgb(172, 79, 35);}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node rect,#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].cluster rect,#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node polygon{stroke:url(#chatgpt-mermaid-\_r_2le\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].swimlane.cluster rect{filter:none;}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node path{stroke:url(#chatgpt-mermaid-\_r_2le\_-gradient);stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node .outer-path{filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node .neo-line path{stroke:rgb(172, 79, 35);filter:none;}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node circle{stroke:url(#chatgpt-mermaid-\_r_2le\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].node circle .state-start{fill:#000000;}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].icon-shape .icon{fill:url(#chatgpt-mermaid-\_r_2le\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2le\_ [data-look="neo"].icon-shape .icon-neo path{stroke:url(#chatgpt-mermaid-\_r_2le\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2le\_ .node text{font-size:14px;font-weight:600;letter-spacing:normal;fill:rgb(255, 183, 144);}#chatgpt-mermaid-\_r_2le\_ .edgeLabels text{font-size:13px;font-weight:600;letter-spacing:-0.08px;fill:rgb(255, 183, 144);}#chatgpt-mermaid-\_r_2le\_ .node tspan[font-weight="normal"],#chatgpt-mermaid-\_r_2le\_ .edgeLabels tspan[font-weight="normal"]{font-weight:600;}#chatgpt-mermaid-\_r_2le\_ .edgeLabel .label rect{opacity:1;rx:13px;ry:13px;fill:rgb(40, 17, 5);stroke:rgb(92, 56, 31);stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .node rect,#chatgpt-mermaid-\_r_2le\_ .node circle,#chatgpt-mermaid-\_r_2le\_ .node ellipse,#chatgpt-mermaid-\_r_2le\_ .node polygon,#chatgpt-mermaid-\_r_2le\_ .node path{fill:rgb(74, 34, 6);stroke:rgba(255, 255, 255, 0.1);stroke-width:1px;}#chatgpt-mermaid-\_r_2le\_ .node rect{rx:16px;ry:16px;}#chatgpt-mermaid-\_r_2le\_ .node.mermaid-decision .label-container{fill:rgb(40, 17, 5);stroke:rgb(92, 56, 31);stroke-dasharray:2,2;}#chatgpt-mermaid-\_r_2le\_ .edgePaths .flowchart-link{stroke:rgb(175, 175, 175);stroke-width:1px;stroke-linecap:round;stroke-linejoin:round;}#chatgpt-mermaid-\_r_2le\_ .marker{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2le\_ :root{--mermaid-font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";}Phase 1: Architecture reviewPhase 2: Core modelPhase 3: ValidationPhase 4: Basic operationsPhase 5: NormalizationPhase 6: SizingPhase 7: EmbeddingPhase 8: FloatingPhase 9: Engine integrationPhase 10: History andpersistencePhase 11: Frontend contractPhase 12: Verification

My recommendation: implement Phases 1–5 as the first milestone. They establish a valid, testable docking model and the core operations before the more complex host lifecycle and floating behavior are introduced. Then add sizing, embedding, and floating support, integrating each through the same operation and validation mechanisms.

This keeps the first milestone useful on its own and reduces the risk of implementing advanced docking behavior on top of an unstable structural model.