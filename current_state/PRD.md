# Docking Environment — Product Requirements Document (PRD)

**Project:** AppShell  
**Subsystem:** Domain Foundation  
**Environment:** Docking Environment  
**Document type:** Product Requirements Document and proposed design baseline  
**Status:** Proposed — not implemented or finalized  
**Version:** 0.1  
**Primary objective:** Establish a reusable, headless docking-layout environment as a new built-in Domain Foundation environment.

---

## 1. Executive Summary

The Docking Environment will provide a headless domain model for managing the structural arrangement of application work areas. It will allow an application to represent, validate, modify, serialize, and restore a docking layout without coupling its core logic to a specific rendering framework or input system.

The environment is intended for applications that need flexible workspaces, including editors, creative applications, development tools, dashboards, and other applications in which users can arrange multiple work areas.

The environment will establish rules and operations for:

- Arranging docks horizontally and vertically.
- Grouping docks into tab groups.
- Moving individual docks and entire dock groups.
- Embedding docks into eligible hosts.
- Managing edge-docked and floating placements.
- Resizing adjacent regions and maintaining sizing preferences.
- Collapsing and restoring eligible docks.
- Validating structural changes and preserving layout integrity.
- Supporting layout persistence, restoration, and reversible operations through AppShell's existing mechanisms where applicable.

The environment will not render a user interface, manage browser elements, interpret pointer coordinates, or implement a particular drag-and-drop framework. Those responsibilities belong to a frontend or application integration layer.

### 1.1 Product vision

Establish a reusable docking model in which **the logical layout is the source of truth, structural operations are independent of presentation, and all accepted changes preserve the layout's defined invariants**.

### 1.2 Core architectural principle

The Docking Environment defines what a docking layout means and how it can change. The frontend defines how that layout is presented and how users request changes.

### 1.3 Document conventions

- **MUST** indicates a required behavior for the proposed baseline.
- **SHOULD** indicates a recommended behavior that may be adjusted with justification.
- **MAY** indicates an optional capability.
- **Deferred** identifies functionality not required for the initial version.
- All requirements in this document describe a proposed design contract, not an existing implementation.

---

## 2. Problem Statement

Applications with multiple work areas often need users to rearrange panels to match their workflows. A docking system must manage more than the visible position of a panel: moving a dock can alter parent-child relationships, create or remove groups, affect neighboring sizes, change active tabs, and alter the structure that must later be restored.

Implementing these behaviors directly in rendering code creates several risks:

1. Layout rules become coupled to specific UI components.
2. Moving nested groups can accidentally lose or rearrange descendants.
3. Different interactions may implement conflicting structural rules.
4. Invalid layouts may be created during failed operations.
5. Resizing may update only visual styles without updating the logical state.
6. Persistence becomes dependent on rendered elements rather than domain state.
7. Supporting another frontend may require rewriting the docking logic.
8. Structural operations become difficult to test without running a graphical interface.

The Docking Environment addresses these problems by centralizing the layout model, structural operations, validation rules, and persistent layout preferences in a headless environment.

---

## 3. Goals and Non-Goals

### 3.1 Goals

**G-01 — Headless operation**

The core layout model and structural operations MUST function without a DOM, CSS, browser events, or a specific rendering framework.

**G-02 — Structured layouts**

The environment MUST represent relationships, ordering, and placement explicitly rather than treating every dock as an independent visual rectangle.

**G-03 — Safe structural changes**

Operations MUST preserve the layout's invariants. A rejected or failed operation MUST NOT leave the layout partially modified.

**G-04 — Multiple docking behaviors**

The environment MUST support splitting, tab grouping, and embedding as distinct structural behaviors.

**G-05 — Group preservation**

Moving a container MUST normally preserve its descendants and internal arrangement.

**G-06 — Explicit sizing state**

Sizing preferences and other layout properties that need to survive reconstruction MUST be represented in logical state.

**G-07 — Extensible application integration**

Applications MUST be able to identify docks, request supported operations, inspect the resulting layout, and integrate their own panel content through a defined boundary.

**G-08 — AppShell integration**

The environment MUST follow the established patterns for built-in Domain Foundation environments, including module registration, state ownership, operation execution, error handling, and tests.

**G-09 — Verifiability**

Core behaviors MUST be testable independently of frontend rendering.

**G-10 — Recoverable layouts**

The model SHOULD support validation and restoration of saved layouts and SHOULD remain compatible with AppShell's existing undo, redo, and serialization mechanisms where supported.

### 3.2 Non-goals

The initial environment will NOT:

- Render panels, headers, tabs, splitters, or floating windows.
- Manage DOM nodes, CSS classes, or visual stacking.
- Detect pointer positions or calculate hit-test regions from screen coordinates.
- Implement browser drag events or pointer capture.
- Define the visual appearance of docking controls.
- Require a particular frontend framework.
- Provide a panel-content implementation for host applications.
- Implement cross-process or cross-window docking.
- Require an AI agent, MCP server, or network service.
- Require a separate docking plugin system merely to use the built-in environment.
- Automatically merge all adjacent containers with the same orientation.
- Guarantee arbitrary custom dock-category support in the first version.
- Require shared-junction resizing, touch interactions, or keyboard-only docking in the first version.

---

## 4. Users and Use Cases

### 4.1 Primary users

**Application developers**

Developers integrate the environment into applications that require a structured workspace.

**Frontend implementers**

Developers build rendering and interaction adapters that translate user input into domain operations.

**Host applications**

Applications supply panel identities, content ownership, configuration, and any application-specific permissions or policies.

**End users**

End users arrange panels through the frontend. They are not expected to interact with the headless model directly.

### 4.2 Primary use cases

1. A developer creates a default layout containing an editor and an inspector.
2. A user splits an editor area to show another work area beside it.
3. A user combines two work areas into a tab group.
4. A user moves an entire group to another part of the layout.
5. A user embeds an inspector into the edge of an eligible host.
6. A user opens a floating dock inside a host.
7. A user resizes neighboring areas.
8. A user collapses a panel and later restores it.
9. An application saves the layout and restores it in a later session.
10. A developer tests structural operations without launching a graphical interface.
11. An application rejects an invalid docking request without damaging the current layout.
12. A frontend reconstructs the visible interface from the logical layout after a render refresh.

---

## 5. Terminology

| Term | Definition |
|---|---|
| Dock | A logical unit participating in the docking layout. It may represent a panel or a structural group. |
| Panel | An individual work area represented by a dock. |
| Container | A dock that organizes other docks. |
| Split container | A container arranging children horizontally or vertically. |
| Tab group | A container retaining multiple docks while identifying one active member. |
| Embed host | An eligible dock that retains its own work area while providing placements for additional docks. |
| Root | The designated top-level layout node. |
| Parent | The container or host that owns a dock's placement. |
| Child | A dock placed inside a parent container or host. |
| Slot | A logical position in which a dock may be placed. |
| Edge region | A host placement associated with the top, bottom, left, or right side. |
| Floating dock | A dock placed within a host's floating-dock collection rather than an ordinary split or tab position. |
| Source | The dock being moved or otherwise acted upon. |
| Target | The dock or host against which an operation is requested. |
| Drop region | A frontend-detected region communicating the intended placement direction or center interaction. |
| Docking mode | The selected interpretation of a docking request, such as splitting, tab grouping, or embedding. |
| Layout operation | A domain-level request to create or change layout state. |
| Structural normalization | The cleanup of eligible groups made redundant by an operation. |
| Persistent state | Layout state intended to survive reconstruction or serialization. |
| Transient state | Temporary interaction state that need not be stored as part of the logical layout. |
| Derived geometry | Visual dimensions and positions calculated by a frontend from the logical layout and available space. |

---

## 6. Product Architecture

### 6.1 Responsibility boundaries

The proposed architecture separates the system into the following responsibilities.

#### A. Docking model

Owns the logical representation of docks, dock categories, hierarchy, placement, active-tab selection, sizing preferences, and relevant layout state.

#### B. Structural operation system

Implements creation, lookup, movement, insertion, grouping, embedding, removal, and structural normalization.

#### C. Validation system

Checks structural invariants, placement compatibility, identity uniqueness, and operation preconditions.

#### D. Sizing system

Defines persistent sizing preferences, size constraints, and the logical effects of resizing.

#### E. Operation execution boundary

Provides the interface through which requested operations are validated and applied. It integrates with the Domain Engine's established execution and error-handling conventions.

#### F. Persistence integration

Supports serializing and restoring the logical layout if the selected AppShell serialization conventions permit it. Imported state must be validated before being trusted.

#### G. Frontend adapter

Interprets user interactions and translates them into domain requests. It also renders the model, measures visual geometry, and displays operation feedback.

#### H. Demonstration and test interface

Provides a sample environment for validating behavior. It MUST NOT be the only way to create or manipulate docking layouts.

### 6.2 Dependency rules

1. The logical model MUST NOT depend on frontend rendering.
2. Structural operations MUST NOT require pointer coordinates or rendered rectangles.
3. The frontend MAY depend on the docking environment's public API.
4. The docking environment MUST NOT depend on the demo application.
5. Application-owned content MUST remain under application ownership.
6. Persistence MUST operate on logical state rather than serialized DOM structure.
7. Integration with the Domain Engine MUST follow existing environment conventions rather than introduce a parallel environment lifecycle.

### 6.3 Built-in environment integration

The Docking Environment will be registered as a new built-in Domain Foundation environment.

The implementation MUST follow the existing project's conventions for:

- Environment module registration.
- Environment state initialization.
- Public accessors.
- Operation execution.
- Error propagation.
- Serialization and deserialization, where applicable.
- Undo and redo integration, where supported.
- Test organization and public API exposure.

Names such as `docking()`, `execute_docking(...)`, and `DockingOperation` are candidate API names only. The final names and signatures MUST be aligned with the actual existing environment implementations before implementation begins.

The environment MUST NOT create a separate, competing Domain Engine or bypass its established operation-execution mechanisms.

---

## 7. Domain Model Requirements

### 7.1 Dock identity

**DM-01:** Every dock MUST have a stable identity.

**DM-02:** Identities MUST be unique within the layout's defined identity scope.

**DM-03:** Moving or structurally reparenting a dock MUST NOT change its identity.

**DM-04:** Identity collisions in imported or externally supplied layouts MUST be detected before the layout is accepted.

**DM-05:** The identity-generation policy MUST be compatible with AppShell's established conventions. The environment must define whether IDs are generated by the environment, supplied by the host, or supported through both paths.

A dock's identity MUST NOT depend on its rendered element, screen position, current parent, or array index.

### 7.2 Dock categories

The initial design recognizes five categories.

#### Panel

Represents an individual work area.

A panel may have application-defined content identity or metadata, but the environment does not own or render that content.

A panel MUST NOT contain ordinary child docks unless it has been explicitly promoted to a supported host category.

#### Row container

Arranges an ordered sequence of child docks horizontally.

The model MUST preserve child ordering and sizing preferences.

#### Column container

Arranges an ordered sequence of child docks vertically.

The model MUST preserve child ordering and sizing preferences.

#### Tab group

Retains multiple docks in a shared position and identifies one active member.

The model MUST preserve group membership, tab order, and active-tab validity.

#### Embed host

Retains its own work-area identity and provides additional placements for embedded or floating docks.

A host MUST preserve its underlying work area when a dock is embedded into it.

The host's ordinary content is an application-owned work area, not a frontend object owned by the docking environment.

### 7.3 Structural versus content identity

The environment MUST distinguish the identity of a dock from the application content associated with it.

For example, a dock may remain the same logical dock while the frontend reconstructs its visual representation. Likewise, the host application may associate a panel with an editor, inspector, canvas, or other content without requiring the environment to know how that content is rendered.

### 7.4 Root

**DM-06:** Each valid layout MUST have exactly one designated root.

The root MUST remain structurally valid throughout accepted operations.

The proposed baseline is a structural root that organizes the primary layout and does not expose ordinary panel controls. The root should not be removed by structural normalization.

The following conditions require explicit handling:

- Empty initial layout.
- Root containing one child.
- Root containing multiple children.
- Removal of the root's final child.
- Restoration of a saved layout whose root is invalid.

The initial implementation SHOULD use an explicit empty-layout state or a valid empty root representation, consistent with AppShell's existing type and lifecycle conventions. It MUST NOT create a fake user-facing panel merely to satisfy an internal structural requirement.

### 7.5 Placement slots

**DM-07:** Every dock MUST have a valid logical placement.

The model must represent at least:

- An ordered child position in a row container.
- An ordered child position in a column container.
- Membership in a tab group.
- An occupied edge region of an embed host.
- An entry in a host's floating-dock collection.

The internal representation MAY use a tree, explicit slot structures, or a combination. The chosen representation MUST make every supported placement discoverable and verifiable.

### 7.6 Proposed relationship model

The initial model SHOULD use a hierarchical structure for ordinary split and tab relationships, with explicit placement collections for host edge regions and floating docks.

This is preferred over relying exclusively on a simple tree because host placements have distinct semantics and may not behave like ordinary ordered children.

A floating placement SHOULD be able to preserve a dock subtree intact. This means that moving a floating group need not flatten or recreate its descendants.

### 7.7 Model validation

The model MUST support validation of:

- Identity uniqueness.
- Root validity.
- Parent-child compatibility.
- Placement-slot compatibility.
- Absence of invalid cycles.
- Valid active-tab selection.
- Valid host placement ownership.
- Valid sizing state.
- Valid references to existing docks.

Validation MUST inspect all supported placement categories, not only ordinary container children.

---

## 8. Structural Operations

### 8.1 Operation contract

Every structural operation MUST define:

1. Its requested action.
2. Its required input.
3. Its preconditions.
4. Its supported source and target categories.
5. Its expected structural result.
6. Its validation rules.
7. Its possible failure conditions.
8. Its effects on sizing and active selection.
9. Its structural normalization behavior.
10. Its relationship to undo and redo.

An operation MUST either commit a valid result or leave the previous layout unchanged.

### 8.2 Operation atomicity

**OP-01:** An operation MUST NOT leave a dock detached after a failed move.

**OP-02:** An operation MUST NOT commit an invalid intermediate structure as the final layout.

**OP-03:** All affected parent relationships, slots, and related layout properties MUST be updated consistently.

**OP-04:** Failed operations MUST provide a meaningful result or error according to AppShell's established error conventions.

The implementation MAY use pre-validation, a reversible change plan, a temporary candidate state, or another mechanism to achieve these guarantees.

### 8.3 Create

The environment MUST support creation of a valid initial layout and the creation of supported dock categories where allowed.

Creation operations MUST establish valid identity and placement information.

Creation of a structural container MUST define the initial child arrangement and sizing state.

The initial version SHOULD provide a way to create an empty layout and subsequently populate it without depending on a graphical interface.

### 8.4 Find and inspect

The environment MUST support finding docks by identity and determining their current placement.

Inspection MUST work across:

- Split-container children.
- Tab-group members.
- Embedded edge regions.
- Floating-dock collections.

A missing identity MUST produce a defined not-found outcome rather than silently selecting another dock.

### 8.5 Detach and reattach

Moving a dock normally requires removing it from its current slot and placing it in a destination slot.

The source dock and its descendants MUST remain logically intact unless the requested operation explicitly defines a different structural result.

The operation MUST resolve the destination against the correct candidate layout and MUST handle changes to the source's former parent.

### 8.6 Split or insert

A split operation places a source beside a target along a selected direction.

Supported directions:

- Left.
- Right.
- Top.
- Bottom.

The operation SHOULD insert into an existing compatible container when the target's placement and the requested direction permit it. Otherwise, it SHOULD create a new row or column container around the relevant docks.

The operation MUST define:

- Child ordering.
- Initial sizing distribution.
- Behavior when the target is a tab-group member.
- Behavior when the target belongs to an embedded region.
- Behavior when the source is already in the requested container.
- Cleanup of the source's former parent.
- Rejection of invalid ancestor/descendant moves.

The operation MUST NOT silently merge arbitrary same-direction containers merely because their orientations match.

### 8.7 Tab grouping

A tab-group operation places the source and target in a common tab group.

If a suitable tab group already exists, the source SHOULD join that group. Otherwise, a new tab group SHOULD be created around the source and target.

The operation MUST define:

- The resulting tab order.
- Which tab becomes active.
- How an existing tab group is reused.
- How a source already in the group is handled.
- How the original parent is normalized.
- How sizing preferences transfer to the new group.
- Which dock categories may be tab-group members.

**Proposed baseline:** a newly added dock becomes the active member unless the operation explicitly requests another selection or the host application supplies a different policy.

The operation MUST reject or explicitly handle attempts to create invalid self-membership or recursive structures.

### 8.8 Embedding

Embedding places a dock in an eligible host's edge region or floating-dock collection.

The host MUST retain its underlying work area.

The initial version SHOULD support one occupied dock subtree per edge region:

- Top.
- Bottom.
- Left.
- Right.

The initial version SHOULD also support multiple floating dock entries per host, subject to the chosen identity and placement rules.

The operation MUST define:

- Whether the target is an eligible host.
- Whether the host may be promoted to host status.
- Whether the destination region is already occupied.
- How size preferences are established.
- How the source's previous parent is normalized.
- How floating placement is represented.
- What happens when an embedding request fails.

**Proposed baseline for occupied regions:** reject an attempt to occupy an already occupied edge region unless the operation explicitly includes a replacement or exchange policy. The system MUST NOT silently discard the existing dock.

### 8.9 Host promotion and demotion

An eligible dock MAY be promoted to embed-host status when a valid embedding operation requires it.

Promotion MUST preserve the dock's identity and underlying work area.

The implementation MUST define which category-specific properties survive promotion and how the host's placement state is initialized.

**Proposed baseline:** when a host has no embedded or floating docks remaining, it SHOULD be demoted to its prior ordinary category if the host was created solely to support embedding. An explicitly configured persistent host MAY remain a host.

The model MUST distinguish a temporary promotion from an intentionally persistent host.

### 8.10 Swap operation

A center drop could mean swapping positions, joining a tab group, or another operation. These meanings MUST NOT be inferred ambiguously from pointer location alone.

**Initial-version decision:** position swapping is not a required built-in operation in version 1. A center drop will be interpreted by the selected operation mode. A future swap operation may be added after its sizing and compatibility semantics are defined.

### 8.11 Remove and close

The environment MUST support removing a dock from the logical layout.

The operation MUST define whether removal means:

- Removing a dock from its current placement.
- Removing a dock and its entire descendant subtree.
- Removing a tab member while retaining the tab group.
- Removing an embedded or floating dock from its host.

The host application remains responsible for the lifetime of actual panel content unless a higher-level application contract says otherwise.

Closing a dock MUST NOT automatically destroy application-owned content unless that behavior is explicitly requested by the host application.

### 8.12 Structural normalization

After a successful operation, eligible parent containers SHOULD be normalized.

Proposed rules:

1. Remove eligible non-root containers that have no children.
2. Replace eligible one-child containers with their remaining child.
3. Preserve groups with meaningful independent semantics.
4. Never remove the designated root.
5. Do not flatten same-direction containers without an explicit normalization rule.
6. Transfer or recalculate sizing preferences when normalization changes a parent's children.
7. Maintain valid tab selection after membership changes.
8. Preserve host state when it is still semantically required.

Normalization MUST be deterministic for the same initial layout and operation.

---

## 9. Sizing and Geometry

### 9.1 Separation of logical sizing and rendered geometry

The environment MUST distinguish logical size preferences and constraints from actual rendered dimensions.

Logical sizing state may include relative weights, minimum dimensions, optional maximum dimensions, and floating-position preferences.

The frontend calculates actual dimensions from the available space and the logical state.

The environment MUST NOT require a browser measurement to validate a structural operation.

### 9.2 Split sizing

**Proposed baseline:** row and column children use relative sizing weights.

Weights express the intended distribution of available space among siblings. The environment MUST define how weights are initialized, updated, normalized, and transferred during structural changes.

The model MUST NOT mix relative weights and absolute pixel values without an explicit conversion policy.

### 9.3 Bounds

The environment SHOULD support minimum dimensions for panels, containers, embedded regions, and floating docks where appropriate.

Maximum dimensions MAY be supported where needed.

The host application or frontend may provide available geometry and constraints, but these must be translated into a consistent operation or layout policy.

If a resize request would violate a bound, the operation MUST either reject it or apply a defined constraint policy. It MUST NOT silently produce invalid sizing state.

### 9.4 Splitter resizing

The environment MUST provide a way to update the logical sizing preferences of adjacent regions.

A resize operation MUST identify the affected regions and define how the resulting preferences are calculated.

The frontend is responsible for measuring the pointer movement and translating it into a requested size change.

The initial implementation SHOULD derive each update from the gesture's initial state and total displacement rather than repeatedly accumulating rounded incremental changes.

The model MUST define how resizing behaves when:

- One neighboring region reaches its minimum.
- Both neighboring regions have constraints.
- The container has more than two children.
- The total available size is insufficient to satisfy all minimums.
- The resize is cancelled before the frontend commits the change.

### 9.5 Embedded-region resizing

The environment MUST represent persistent sizing preferences for occupied host regions.

A resizing operation MUST update the relevant region state coherently without invalidating the host's other placements.

The model MUST define how top/bottom regions interact with left/right regions and how the available area is allocated among all occupied regions and the underlying host content.

The frontend decides the actual geometry and hit-testing behavior.

### 9.6 Floating docks

Floating docks MUST have logical placement and sizing state sufficient for the frontend to reconstruct their position and dimensions within the host.

The initial version SHOULD constrain floating docks to their host's logical bounds, with the frontend responsible for applying the resulting geometry.

Floating placements SHOULD preserve group structure where supported.

The design MUST establish whether floating bounds refer to the dock body, the complete visible frame, or another consistent logical rectangle.

### 9.7 Collapse and restore

The environment SHOULD support a logical collapsed state for eligible docks.

Collapse MUST preserve sufficient state to restore the dock.

The implementation MUST define which categories can collapse, how collapsed sizing preferences are retained, and how collapse interacts with parent constraints.

Collapse MUST NOT make a dock permanently unreachable.

The frontend determines the visual presentation of collapsed docks.

### 9.8 Shared-junction resizing

Shared-junction resizing, which changes multiple connected boundaries in one gesture, is deferred from the initial required scope.

The model SHOULD avoid assumptions that would make future support impossible, but it does not need to implement junction detection or multi-boundary interaction in version 1.

---

## 10. Interaction Contract

The headless environment does not implement drag-and-drop, but its public operation contract must support a frontend that does.

### 10.1 Drag lifecycle

The frontend's typical interaction lifecycle is:

1. Identify the source dock or group.
2. Start a transient drag interaction.
3. Determine eligible targets.
4. Identify the current target and drop region.
5. Determine the requested docking mode.
6. Submit an operation request.
7. Display the accepted layout or report a rejection.
8. Clear transient interaction state after success, cancellation, or failure.

The environment MUST NOT require these frontend stages to occur in a particular event framework.

### 10.2 Drop regions

The frontend MAY expose five conceptual regions:

- Top.
- Bottom.
- Left.
- Right.
- Center.

A region indicates the user's intended placement. It does not itself define the complete operation.

The selected operation mode and region MUST together determine the requested result.

### 10.3 Docking modes

The initial interaction model SHOULD distinguish:

- **Split/insert:** places the source beside the target.
- **Tab grouping:** places the source and target in a shared tab group.
- **Embedding:** places the source in a host's edge or floating slot.

Unsupported combinations MUST produce an explicit rejection or a documented alternative. They MUST NOT silently execute a different operation.

### 10.4 Source and target eligibility

The frontend SHOULD avoid presenting invalid targets.

However, frontend eligibility checks are not a replacement for domain validation. Every submitted operation MUST be validated by the environment.

The environment MUST reject invalid self-targeting and operations that would make a source its own descendant.

### 10.5 Cancellation and failure

Cancellation, focus loss, pointer exit, invalid drops, and rejected operations are primarily frontend interaction concerns.

The frontend MUST clear transient drag state regardless of whether the operation succeeds.

A rejected operation MUST leave the logical layout unchanged.

### 10.6 Input accessibility

The environment MUST NOT encode assumptions that only mouse-based interaction can request operations.

Keyboard and touch support are deferred as full interaction features, but the operation API SHOULD be usable by other input methods without changing the core structural rules.

---

## 11. State Management

### 11.1 Persistent logical state

The model SHOULD distinguish state intended to survive rendering, reconstruction, and serialization, including:

- Dock identities and categories.
- Parent-child relationships and ordering.
- Tab membership and active selection.
- Embedded and floating placements.
- Relative sizing preferences.
- Configured size bounds where appropriate.
- Persistent collapse preferences.
- Host promotion or persistence state where needed.
- Schema/version information when required by the serialization design.

The exact fields MUST be derived from the final model rather than added solely to support one frontend's rendering implementation.

### 11.2 Transient interaction state

Transient interaction state may include:

- Active drag source.
- Current pointer target.
- Highlighted drop region.
- Temporary docking mode selection.
- Temporary overlay visibility.
- Hover and reveal state.
- In-progress resize gesture state.

These values SHOULD remain outside the persistent layout model unless a specific value has a separately defined domain meaning.

### 11.3 Derived visual state

The following SHOULD be derived by the frontend:

- DOM or widget instances.
- CSS classes.
- Pixel rectangles.
- Pointer hit-test results.
- Overlay positions.
- Visual stacking and z-order.
- Animated transitions.
- Hover effects.

The logical model MUST remain valid even when the frontend destroys and rebuilds its rendered representation.

### 11.4 Change boundaries

Structural operations, persistent sizing changes, and transient interaction changes MUST have explicit commit boundaries.

The implementation SHOULD avoid updating persistent layout state for every temporary visual effect.

A resize gesture may involve transient intermediate measurements followed by a committed logical sizing change. The exact commit strategy must be consistent with AppShell's operation and history conventions.

---

## 12. Persistence and Restoration

### 12.1 Serialization

The environment SHOULD support serialization through AppShell's established serialization approach where available.

The serialized form MUST represent logical layout state rather than the rendered interface.

It SHOULD contain enough information to reconstruct:

- The root.
- Dock identities and categories.
- Parent-child relationships and ordering.
- Active tab.
- Embedded and floating placements.
- Persistent sizing preferences.
- Persistent collapse state.
- Other state required by the approved model.

### 12.2 Schema versioning

If layouts are persisted across versions, the format MUST have a defined compatibility strategy.

The design MUST specify how unsupported or outdated layout versions are handled. An implementation MUST NOT assume that every saved layout was produced by the current version.

### 12.3 Restoration

Restoration MUST validate incoming layout data before accepting it as trusted state.

A restoration attempt MUST detect malformed relationships, duplicate identities, unsupported categories, invalid slots, and invalid active-tab references.

The environment MUST define whether an invalid layout is rejected completely or repaired through a documented migration or recovery process.

The initial version SHOULD prefer rejection with a meaningful error over silent, undocumented repair.

### 12.4 Content references

The environment may preserve application-provided content identifiers or metadata where required, but it MUST NOT attempt to serialize arbitrary frontend objects as part of the layout.

The host application remains responsible for restoring actual panel content.

---

## 13. Undo and Redo

Undo and redo are important integration considerations because structural changes may affect multiple parts of the layout.

### 13.1 Requirements

**UR-01:** The environment MUST follow AppShell's existing history and reversibility conventions.

**UR-02:** A reversible operation MUST preserve sufficient information to restore the prior logical layout.

**UR-03:** Reversing an operation MUST restore affected relationships, ordering, active selection, and sizing preferences to the appropriate prior values.

**UR-04:** A failed operation MUST NOT create a successful history entry.

**UR-05:** Undo and redo MUST preserve layout validity.

### 13.2 Candidate reversible operations

The history design should consider:

- Moving a dock.
- Splitting a region.
- Creating or modifying a tab group.
- Embedding a dock.
- Promoting or demoting a host.
- Removing a dock.
- Changing active-tab selection, if that action is intended to be undoable.
- Resizing a region.
- Collapsing or restoring a dock.

The exact granularity of history entries must follow AppShell's established behavior.

The implementation SHOULD avoid recording every intermediate pointer movement as a separate persistent resize operation when the frontend can commit a completed gesture as one logical action.

---

## 14. Errors and Failure Behavior

The environment MUST define meaningful failure outcomes for at least the following conditions:

- Dock identity not found.
- Duplicate identity.
- Invalid root.
- Unsupported source category.
- Unsupported target category.
- Invalid destination slot.
- Self-targeting operation.
- Ancestor/descendant cycle.
- Occupied edge region.
- Missing or ineligible host.
- Invalid active-tab selection.
- Invalid sizing state.
- Malformed imported layout.
- Unsupported serialized schema.
- Failed operation precondition.
- Invalid structural result.

Errors MUST follow AppShell's established error conventions. The names and representation of individual error variants must be aligned with the actual repository before implementation.

The environment SHOULD distinguish invalid requests from unexpected execution failures where the existing architecture supports that distinction.

Failure handling MUST NOT leave the model partially modified.

---

## 15. Public API Requirements

The environment's public API must be designed after reviewing the actual conventions of AppShell's existing built-in environments.

The API SHOULD provide capabilities equivalent to the following conceptual groups.

### 15.1 State access

- Access the current docking layout.
- Find a dock by identity.
- Inspect its category and placement.
- Inspect the root and relevant descendants.
- Inspect tab-group membership and active selection.
- Inspect host edge and floating placements.
- Inspect sizing preferences.

### 15.2 Operation execution

- Create or initialize a valid layout.
- Create supported docks.
- Move a dock.
- Split or insert a dock.
- Add a dock to a tab group.
- Embed a dock.
- Change active-tab selection.
- Update sizing.
- Collapse or restore a dock.
- Remove a dock.
- Validate the current layout.
- Restore a layout from a validated representation.

### 15.3 API design principles

1. Public operations MUST communicate intent rather than visual implementation details.
2. Operations MUST use stable identities rather than rendered-element references.
3. Structural changes MUST be routed through the environment's operation boundary.
4. Invalid operations MUST return a defined failure result.
5. The public API MUST not expose mutable internal structures in a way that bypasses validation.
6. Public names, visibility, and method signatures MUST match AppShell's existing conventions.
7. The initial API SHOULD remain as small as practical while still covering the approved behavior.

This section describes required capabilities, not final Rust signatures.

---

## 16. Proposed Module Organization

The following is a candidate module structure. It must be reconciled with the source organization used by existing Domain Foundation environments before implementation.

- `environments/docking/mod.rs`
  - Public exports and environment module boundary.

- `environments/docking/model.rs`
  - Top-level logical layout representation.

- `environments/docking/dock.rs`
  - Dock identity, categories, and shared dock information.

- `environments/docking/layout.rs`
  - Root, hierarchy, traversal, and layout inspection.

- `environments/docking/slot.rs`
  - Ordinary child placements, edge slots, and floating placements.

- `environments/docking/operation.rs`
  - Public operation definitions and operation-related result types.

- `environments/docking/operations.rs`
  - Structural operation execution and normalization.

- `environments/docking/sizing.rs`
  - Sizing preferences, bounds, and resize transformations.

- `environments/docking/validation.rs`
  - Structural and state invariants.

- `environments/docking/error.rs`
  - Environment-specific error definitions.

The final module structure MAY combine files where the existing project favors fewer modules. The responsibilities, rather than the exact filenames, are the important requirement.

The environment's registration and execution integration MUST be implemented according to AppShell's actual Domain Engine patterns.

---

## 17. Non-Functional Requirements

### 17.1 Correctness

The environment MUST prioritize structural correctness over rendering performance.

Every accepted operation MUST produce a valid layout under the approved invariant set.

### 17.2 Determinism

Given the same initial logical state and the same operation request, the structural result SHOULD be deterministic.

Operations MUST NOT rely on DOM ordering, pointer coordinates, or frontend timing.

### 17.3 Testability

The core model and operations MUST be testable without a browser or graphical frontend.

### 17.4 Performance

The implementation SHOULD avoid unnecessary repeated full-layout traversal during a single operation where practical.

No specific performance target is established by this PRD. Benchmarks should be added after the expected maximum layout size and usage patterns are understood.

### 17.5 Maintainability

Structural rules SHOULD be centralized rather than duplicated between operations.

Normalization, validation, identity lookup, and placement compatibility SHOULD have clearly defined responsibilities.

### 17.6 Compatibility

The implementation MUST follow AppShell's established conventions for public API, errors, history, serialization, and environment integration.

### 17.7 Extensibility

The architecture SHOULD allow additional docking behaviors to be introduced through a defined operation boundary.

Adding entirely new dock categories is a separate concern because it affects representation, validation, persistence, and frontend integration. Arbitrary custom categories are not required in the initial version.

### 17.8 Portability

The headless core MUST avoid assumptions specific to a browser or one UI toolkit.

A frontend may have platform-specific behavior, but the logical layout and structural operations should remain reusable.

---

## 18. Testing and Verification Requirements

### 18.1 Model tests

Verify:

- Valid initial layout creation.
- Stable identities after moves.
- Identity uniqueness.
- Root validity.
- Valid parent-child relationships.
- Correct traversal of all supported placement categories.
- Correct lookup of ordinary, embedded, and floating docks.

### 18.2 Structural operation tests

Verify:

- Splitting horizontally and vertically.
- Insertion into compatible containers.
- Creation of new split containers.
- Tab grouping and tab ordering.
- Active-tab selection after membership changes.
- Moving a group without losing descendants.
- Embedding in each supported edge region.
- Floating placement.
- Removal and structural normalization.
- Host promotion and demotion.
- Handling of occupied edge regions.
- Rejection of self-targeting and invalid ancestor/descendant moves.

### 18.3 Atomicity tests

For every operation category:

1. Establish a valid starting layout.
2. Submit an invalid request.
3. Confirm the operation reports failure.
4. Confirm the layout remains unchanged.
5. Confirm no invalid history entry is created.
6. Confirm the layout still passes validation.

### 18.4 Sizing tests

Verify:

- Initial relative weights.
- Weight distribution after splitting.
- Weight transfer after removing a child.
- Resizing with two children.
- Resizing with multiple siblings.
- Minimum-bound enforcement.
- Invalid size rejection.
- Embedded-region sizing.
- Floating-dock sizing.
- Preservation of sizing preferences after movement and restoration.

### 18.5 Persistence tests

Verify:

- Serialization and restoration of representative layouts.
- Preservation of identities and child ordering.
- Preservation of active-tab selection.
- Preservation of embedded and floating placements.
- Preservation of persistent sizing and collapse state.
- Detection of duplicate identities.
- Rejection or documented recovery of malformed layouts.
- Handling of unsupported schema versions.

### 18.6 History tests

Verify that supported reversible operations restore the relevant prior layout state.

Tests must cover nested structures, sizing changes, and normalization, not only simple moves.

### 18.7 Integration tests

Verify:

- Environment registration.
- Initial state construction.
- Access through the Domain Engine.
- Execution through the established operation boundary.
- Error propagation.
- Compatibility with relevant history and serialization mechanisms.
- No requirement for frontend or browser initialization.

### 18.8 Frontend contract tests

Although frontend rendering is outside the environment's scope, an integration prototype SHOULD verify that:

- The frontend can reconstruct a layout from logical state.
- A frontend-detected drop region can be translated into a domain operation.
- A rejected operation does not leave the displayed layout inconsistent with the model.
- A render refresh does not destroy persistent logical state.
- Transient drag state can be cleared independently of operation success.

---

## 19. Acceptance Criteria

The initial environment is not considered complete merely because a demonstration appears to work.

### AC-01 — Built-in environment integration

The Docking Environment is registered and accessible through the Domain Engine using the project's established conventions.

### AC-02 — Valid logical layout

A layout can be created and inspected without a frontend.

### AC-03 — Stable identity

Dock identities remain stable through supported structural operations.

### AC-04 — Split behavior

Horizontal and vertical splitting produce valid structures with defined child order and sizing state.

### AC-05 — Tab grouping

Supported docks can be grouped into tabs, and active selection remains valid after membership changes.

### AC-06 — Embedding

Eligible docks can be embedded in supported host regions without replacing the host's underlying work area.

### AC-07 — Floating placement

Supported docks can be assigned to a host's floating collection while preserving their identity and internal structure.

### AC-08 — Safe movement

Moving a dock or group preserves its intended descendants and does not introduce invalid cycles.

### AC-09 — Structural normalization

Removing or moving docks cleans up eligible redundant containers while preserving meaningful groups and the root.

### AC-10 — Sizing state

Sizing preferences are logical state and remain coherent after supported structural operations.

### AC-11 — Atomic failure

Rejected operations leave the previous valid layout unchanged.

### AC-12 — Validation

The environment detects the approved structural and placement violations.

### AC-13 — Persistence readiness

The model supports the agreed serialization and restoration contract, including validation of imported state, where serialization is enabled.

### AC-14 — History compatibility

Operations designated as reversible work with AppShell's established undo/redo mechanisms.

### AC-15 — Headless testability

The core behavior can be tested without a browser, DOM, or specific UI framework.

### AC-16 — Frontend independence

The core environment contains no required rendering or pointer-event implementation.

### AC-17 — Public API consistency

The public interface and module structure follow the actual conventions of AppShell's existing environments.

### AC-18 — Verification coverage

The structural, operation, error, sizing, and integration tests pass for the supported initial scope.

---

## 20. Implementation Phases

### Phase 1 — Repository alignment and model specification

**Objective:** Finalize the environment's model and its integration boundaries.

Tasks:

- Inspect the current built-in environment implementations.
- Confirm module registration and public API conventions.
- Confirm the operation execution pattern.
- Confirm error and history conventions.
- Define dock categories and identity policy.
- Define the root and placement model.
- Define the invariant set.
- Resolve the representation of embedded and floating placements.
- Define the behavior of empty and single-child containers.

**Exit criteria:** The logical model, invariants, and AppShell integration pattern are approved.

### Phase 2 — Model and validation

**Objective:** Implement the logical representation without frontend interaction.

Tasks:

- Implement dock identities and categories.
- Implement the root and hierarchy.
- Implement explicit placement representation.
- Implement layout inspection and identity lookup.
- Implement structural validation.
- Add model tests.

**Exit criteria:** Valid layouts can be created, inspected, and validated independently of rendering.

### Phase 3 — Structural operations

**Objective:** Implement the core docking transformations.

Tasks:

- Implement creation and movement.
- Implement split/insert.
- Implement tab grouping.
- Implement embedding and floating placement.
- Implement removal and normalization.
- Implement host promotion/demotion where required.
- Implement atomic failure handling.
- Add operation tests.

**Exit criteria:** Every required structural operation has defined behavior, error cases, and passing tests.

### Phase 4 — Sizing and state transitions

**Objective:** Establish persistent sizing behavior.

Tasks:

- Implement relative sizing preferences.
- Define bounds and constraint behavior.
- Implement logical resize operations.
- Implement embedded and floating sizing.
- Implement collapse and restore where included.
- Add sizing and normalization tests.

**Exit criteria:** Supported sizing changes produce valid, persistent logical state.

### Phase 5 — Domain Engine integration

**Objective:** Register and expose the environment using AppShell conventions.

Tasks:

- Register the environment module.
- Integrate the environment's state with the Domain Engine.
- Add the public accessor and execution entry point according to repository conventions.
- Integrate errors.
- Integrate supported history and serialization mechanisms.
- Add Domain Engine integration tests.

**Exit criteria:** The environment works through the normal AppShell environment lifecycle and execution patterns.

### Phase 6 — Persistence and history verification

**Objective:** Verify recovery and reversibility.

Tasks:

- Finalize the serialized layout representation.
- Implement restoration validation.
- Verify schema handling.
- Verify undo/redo for supported operations.
- Test restoration of nested and embedded layouts.

**Exit criteria:** Supported layouts can be restored safely, and reversible operations restore the expected prior state.

### Phase 7 — Frontend adapter prototype

**Objective:** Demonstrate the environment through a real frontend without moving UI responsibilities into the core.

Tasks:

- Render the logical structure.
- Translate drop regions into operation requests.
- Implement drag overlays and interaction feedback.
- Implement splitters and resize input.
- Handle cancellation and failed operations.
- Verify that the rendered state follows the logical model.

**Exit criteria:** A demonstration frontend can manipulate the layout through the public API without owning the structural rules.

---

## 21. Risks and Mitigations

### R-01 — Ambiguous layout representation

**Risk:** Embedded and floating placements may not fit naturally into ordinary parent-child relationships.

**Mitigation:** Define the placement model before implementing operations. Require all placement categories to participate in lookup and validation.

### R-02 — Incorrect structural cleanup

**Risk:** Removing redundant groups may accidentally destroy meaningful tab or host semantics.

**Mitigation:** Define normalization eligibility by category and test nested cleanup cases.

### R-03 — Sizing loss during movement

**Risk:** Creating, removing, or moving groups may lose user sizing preferences.

**Mitigation:** Define size-transfer rules for every structural operation before implementing it.

### R-04 — Partial mutations

**Risk:** A failure after detachment could leave the layout invalid.

**Mitigation:** Require atomic operation semantics and test the unchanged-state guarantee for every failure class.

### R-05 — Frontend logic leaking into the core

**Risk:** The environment may begin depending on pointer coordinates, rendered dimensions, or UI element identity.

**Mitigation:** Keep geometry measurement and input interpretation in the frontend adapter. Accept domain-level operation requests instead.

### R-06 — Excessively broad initial scope

**Risk:** Custom categories, shared junctions, complex floating layouts, and multiple input methods could delay the core environment.

**Mitigation:** Deliver the core model, structural operations, validation, and AppShell integration before adding deferred interaction features.

### R-07 — Incompatible AppShell integration

**Risk:** An independently designed environment API could conflict with existing environment conventions.

**Mitigation:** Review actual source implementations before finalizing names, signatures, error types, and history integration.

### R-08 — Persistence incompatibility

**Risk:** Future model changes could invalidate saved layouts.

**Mitigation:** Define the serialized contract and validation policy before treating the format as stable.

---

## 22. Deferred Features

The following capabilities are not required for the initial version unless later promoted into scope:

- Shared-junction resizing.
- Touch-specific docking interaction.
- Full keyboard-only docking.
- Cross-window docking.
- Floating docks outside their host bounds.
- Arbitrary custom dock categories.
- Automatic merging of same-direction containers.
- Built-in swap behavior for center drops.
- Complex conflict-resolution UI for occupied host regions.
- Animated transitions.
- Frontend-specific header and reveal behavior.
- Built-in undo/redo if the underlying AppShell history integration does not support the required behavior directly.

Deferring these features MUST NOT weaken the core invariants or force frontend-specific concepts into the logical model.

---

## 23. Decisions to Confirm Before Implementation

The PRD establishes a recommended baseline, but these decisions should be verified against AppShell's source conventions and the intended initial scope.

1. **Layout representation:** use a hierarchical model with explicit host placement slots.
2. **Root behavior:** maintain one designated structural root and define a valid empty-layout representation.
3. **Occupied edge region:** reject by default rather than silently replacing existing content.
4. **Center drops:** let the selected docking mode determine the operation; do not implement implicit swapping in version 1.
5. **Sizing representation:** use relative weights for split children and define normalization and transfer rules.
6. **Host promotion:** preserve identity and underlying content; distinguish temporary from persistent host status.
7. **Floating groups:** preserve a dock subtree when moving a supported group into a floating placement.
8. **Operation atomicity:** commit only valid results and leave state unchanged on failure.
9. **Undo/redo:** use AppShell's established history contract rather than creating a separate history mechanism.
10. **Serialization:** follow the project's established approach and validate restored layouts.
11. **Public API:** use names and signatures consistent with the actual built-in environment implementations.
12. **Initial scope:** prioritize the headless model, structural operations, sizing state, validation, and Domain Engine integration before implementing frontend interaction.

If any of these decisions conflict with the existing AppShell architecture, the repository's established conventions and the approved project design should take precedence over the illustrative choices in this PRD.

---

## 24. Definition of Done

The Docking Environment is considered ready for its initial release when:

- Its model and invariants are formally established.
- It is registered as a built-in Domain Foundation environment.
- The public API follows AppShell's conventions.
- Required structural operations are implemented.
- Failed operations preserve the previous valid state.
- Sizing behavior and structural normalization are defined and tested.
- Error handling is integrated.
- Persistence and history behavior are implemented to the extent supported by the approved scope.
- Core tests run without a graphical frontend.
- Integration tests verify execution through the Domain Engine.
- A frontend can render and interact with the environment through the defined API.
- Public documentation describes the actual implemented behavior and its limitations.

**Final design principle:** The Docking Environment owns the meaning and validity of the layout. The frontend owns its presentation and interaction. Neither layer should have to duplicate the other layer's responsibilities to keep the workspace consistent.
