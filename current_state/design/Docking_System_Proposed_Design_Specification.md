# Docking System — Proposed Design Specification

> This document describes a **proposed design direction** for a browser-based docking-panel system. It is not a record of a completed implementation. The concepts, constraints, interaction models, data structures, rules, and mechanisms below are candidates to establish during design and implementation. Where a choice remains open, it is identified as a decision to make rather than presented as an existing fact.

---

## Table of Contents

1. Purpose, scope, and design intent
2. Proposed architecture and responsibilities
3. Docking model and terminology
4. Layout representation and structural changes
5. Layout, sizing, and bounds
6. Parenting and docking behaviour
7. Drag-and-drop interaction
8. Headers, controls, and commands
9. Resizing and shared boundaries
10. Rendering, state, and persistence
11. Extensibility and custom docking behaviours
12. Candidate invariants and validation rules
13. Open questions and risks
14. Verification plan
15. Design decisions to establish

---

## 1. Purpose, scope, and design intent

### 1.1 Purpose

The system is intended to provide a flexible docking-panel experience in which users can arrange work areas by moving, grouping, splitting, overlaying, and resizing panels. A layout should be understandable as a structured arrangement rather than merely a collection of unrelated visual elements.

The intended experience includes:

- Splitting a work area horizontally or vertically.
- Grouping work areas into tabs.
- Placing work areas into edge regions of a host.
- Displaying floating work areas inside a host.
- Moving an entire group while retaining its internal arrangement.
- Resizing adjacent regions and, where appropriate, shared junctions.
- Closing or collapsing eligible docks.
- Revealing controls when they are needed without permanently crowding the workspace.
- Allowing additional docking behaviours to be introduced without rewriting the entire interaction system.

These are design goals, not claims that every behaviour is already implemented or finalized.

### 1.2 Scope

The design covers the conceptual layout model, structural operations, interaction flow, rendering responsibilities, sizing behaviour, extensibility, and validation. It does not prescribe a final programming language, file layout, framework, or exact implementation technique.

The initial target is a browser-based interface. Touch input, keyboard-only docking, cross-window docking, and platform-specific behaviour may need separate decisions.

### 1.3 Design principles to consider

The following principles are proposed as a basis for future decisions:

| Principle | Intended meaning |
|---|---|
| Structured layout as the source of truth | The logical layout should be represented independently of the rendered interface. |
| Clear separation of responsibilities | User-interface code should handle presentation and input, while layout operations should work on the logical model. |
| Move groups intact | Moving a parent dock should normally preserve its descendants and internal structure. |
| Avoid unnecessary wrappers | Structural operations should avoid leaving redundant empty or single-child groups unless a group type has a reason to remain. |
| Predictable drop targets | The interface should make clear which docks can receive a drop and what each region means. |
| Extensible docking behaviours | Additional ways of combining docks should be possible through a defined extension boundary. |
| Recoverable state | The logical layout should be suitable for saving, restoring, validating, and potentially undoing changes. |

These principles still need to be approved and translated into explicit acceptance criteria.

### 1.4 Terminology

| Term | Proposed meaning |
|---|---|
| Dock | Any logical unit represented in the layout, including an individual panel or a group. |
| Panel | An individual work area that does not contain other docks in the layout model. |
| Container | A dock that organizes other docks. |
| Root | The top-level layout container. |
| Child / parent | The relationship between a dock and the container that holds it. |
| Slot | A position in which a dock can be placed, such as an ordered child position, an edge region, or a floating position. |
| Source | The dock the user intends to move. |
| Target | The dock or host over which the source is dropped. |
| Drop region | A region of a target that communicates the intended docking result. |
| Docking mode | A selected interpretation of a drop, such as splitting, tab grouping, or embedding. |
| Header / handle | The visible area used to identify, control, or move a dock. |
| Peek / reveal | A temporary or context-sensitive way to expose controls that are otherwise hidden. |
| Host | A dock that provides a space for embedded regions or floating docks. |

Terminology should be reviewed before the public API and documentation are finalized.

---

## 2. Proposed architecture and responsibilities

### 2.1 Conceptual layers

The system may be organized around the following responsibilities:

- **Layout model:** represents docks, their relationships, their ordering, and layout-specific properties.
- **Structural operations:** locate docks, move them between slots, replace redundant groups, exchange positions, and validate the resulting structure.
- **Docking behaviour registry:** identifies the available ways a drop can change the layout and delegates a drop to the selected behaviour.
- **Command boundary:** expresses user intentions as operations without requiring structural logic to manipulate the interface directly.
- **Interaction layer:** handles pointer and keyboard input, drag state, modifier keys, and user feedback.
- **Rendering layer:** turns the logical layout into a visible interface and refreshes it after structural changes.
- **Geometry and resizing layer:** measures visible boundaries and applies size changes.
- **Persistence layer:** stores and restores layout state, subject to a format and compatibility policy that still need to be chosen.
- **Test or demonstration interface:** provides sample panels and controls for evaluating the system; it should not define the core system's only way of creating docks.

The final module boundaries and dependency directions remain to be established.

### 2.2 Separation of responsibilities

A candidate boundary is that structural operations should not depend on specific DOM elements, CSS classes, or pointer events. The interaction layer would interpret gestures and request operations; the structural layer would apply the requested change to the logical layout; and the rendering layer would reflect the result.

This separation is intended to make layout behaviour testable without a browser interface and to leave room for other frontends. The exact interfaces and error-reporting approach have not yet been defined.

### 2.3 State categories

The design should distinguish between:

- **Persistent layout state:** dock identities, hierarchy, ordering, active tab, collapse preferences, sizing preferences, and embedded/floating placement where applicable.
- **Transient interaction state:** an active drag, the current source dock, currently excluded targets, selected docking mode, highlighted region, and temporary reveal state.
- **Rendered state:** DOM elements, measured rectangles, visual classes, overlays, and handles derived from the other state categories.

Which properties belong in each category should be decided explicitly. A property should not silently alternate between being persistent and being only visual.

### 2.4 Demonstration interface

A development or demo page may offer controls for creating sample panels, viewing the current layout structure, and testing interactions. Those facilities should remain separate from the core model so that the library does not depend on a particular demo layout or panel catalogue.

---

## 3. Docking model and terminology

### 3.1 Candidate dock categories

The initial design may support these categories:

- **Panel:** an individual work area.
- **Row container:** arranges child docks horizontally.
- **Column container:** arranges child docks vertically.
- **Tab group:** holds multiple docks while showing one selected member at a time.
- **Embed host:** retains its own work area while providing edge regions and floating positions for additional docks.

The names and exact semantics of these categories are proposed, not yet a finalized public contract.

### 3.2 Shared dock information

Every dock will likely need a stable identity and a way to distinguish its category. Some docks may also carry a label, a content or presentation hint, a preferred size, and a collapsed state.

The design should decide which fields are universal, which belong only to certain dock categories, and which should be supplied by the host application. A dock's identity should remain stable when it is moved or when its visible representation is rebuilt.

### 3.3 Row and column containers

A row or column container is intended to arrange an ordered set of child docks along one primary direction. Adjacent children would be separated by interactive boundaries that allow their relative sizes to change.

The design should establish whether nested containers of the same direction are allowed, whether they may ever be consolidated, and how their sizing preferences behave when a child is moved or removed.

### 3.4 Tab groups

A tab group is intended to retain several docks in one place while exposing one selected dock's content at a time. Its tab strip may be placed along any edge if the chosen interaction and visual design support all four orientations.

The design must define how active selection changes when tabs are added, removed, reordered, or moved; how nested groups appear; and whether non-panel docks may be tabs.

### 3.5 Embed hosts

An embed host is intended to preserve its own underlying work area while supporting additional docks in edge regions and floating positions. Edge regions may be arranged around the host's central content, and floating docks may be moved or resized within the host's available area.

The geometry of intersecting regions, the treatment of unused corner spaces, clipping, and the relationship between host controls and embedded content all require explicit design decisions.

### 3.6 Root behaviour

The root represents the overall layout. A likely design is to keep it as a structural row or column without ordinary panel controls. Whether it may be empty or contain only one child, and how an empty layout should be repaired or displayed, must be decided.

### 3.7 Placement slots

The model will need to represent the possible places a dock can occupy. Candidate slot categories include ordered children of a container, an edge region of an embed host, and an entry in a host's floating-dock collection.

Every structural operation should identify the slot it is changing. The exact representation is an implementation decision.

---

## 4. Layout representation and structural changes

### 4.1 Layout as a logical structure

The layout should be represented in a form that makes parent-child relationships, order, and placement explicit. A tree-like model is a candidate because it can represent nested splits, tab groups, and hosts in one structure.

The final representation must also account for any placements that do not behave like ordinary ordered children, particularly embedded edge regions and floating docks.

### 4.2 Finding a dock

Structural operations will need a reliable way to locate a dock by identity and determine its parent and placement slot. The search should cover every supported placement category, not just ordinary container children.

The system should define how invalid or duplicate identities are detected and what happens when a requested dock cannot be found.

### 4.3 Detaching and reattaching

Moving a dock will generally require removing it from its current slot and placing it into a destination slot. A move should preserve the source dock and its descendants as a unit unless the selected operation explicitly defines a different result.

Because removing a dock may also change its former parent, the destination should be resolved against the updated layout. The design should also address how a failed operation avoids leaving a dock detached or losing part of the layout.

### 4.4 Removing redundant containers

Empty or unnecessary groups can make a layout harder to reason about. A candidate cleanup policy is:

- Remove an eligible non-root group when it has no children.
- Replace an eligible one-child group with its remaining child.
- Preserve groups that have a meaningful independent role or whose semantics require their continued existence.
- Keep the root under a separate policy.

The precise list of eligible group types, how size preferences transfer, and whether cleanup cascades through ancestors are decisions to establish. Automatically flattening same-direction containers should not be assumed without a separate design decision.

### 4.5 Promotion and demotion of hosts

The system may allow an eligible dock to become an embed host when a user places another dock into one of its edge regions or floating positions. If the host no longer contains any embedded or floating docks, it may be restored to its prior category.

The design needs to decide which dock categories can become hosts, which properties survive this transition, whether an empty host can remain intentionally, and how size settings are preserved across transitions.

### 4.6 Swapping positions

A centre drop may be designed to exchange the positions of two docks rather than create a new group. The behaviour must define whether it is available for every slot type, how sizing preferences are handled, and how to avoid exchanging a dock with one of its own descendants.

### 4.7 Structural validation

Before a layout change is accepted, the system should be able to verify relevant conditions, including identity uniqueness, valid parent-child relationships, supported slot types, and the absence of invalid cycles. Validation boundaries and recovery behaviour remain to be designed.

---

## 5. Layout, sizing, and bounds

### 5.1 Directional layout

Row containers are intended to arrange children from left to right; column containers from top to bottom. The rendered layout should provide a visible and usable boundary between adjacent children.

Exact spacing, border thickness, minimum sizes, and visual treatment should be established by the design system rather than treated as intrinsic properties of the logical model.

### 5.2 Sizing model

A candidate approach is to store relative size preferences for children within a row or column. This would allow the available space to be distributed proportionally while still supporting user-driven resizing.

The design must choose how these preferences are represented and normalized, how they interact with minimum sizes, and how they behave when the number of siblings changes. It should avoid mixing relative weights and pixel measurements without a defined conversion policy.

### 5.3 Minimum and maximum bounds

Panels, containers, embedded regions, and floating docks may need different minimum dimensions. The design should define which limits are mandatory, which can be configured by the host application, and whether maximum bounds are required.

Resizing should behave predictably when a requested movement would violate a bound. Possible policies include stopping at the nearest valid position or clamping the movement to the limit; this choice remains open.

### 5.4 Splitters

Splitters are intended to let users adjust the relative sizes of adjacent children. The design should specify how the affected neighbours are identified, how pointer movement translates into size changes, how bounds are enforced, and when changes are committed to persistent layout state.

The chosen approach should avoid cumulative drift during a drag and should behave consistently across repeated resizes.

### 5.5 Collapse behaviour

Collapse is intended to temporarily reduce a dock's visible footprint while keeping a way to restore it. The design must establish which dock categories can collapse, what remains visible, and whether collapse is supported in every parent context.

It should also decide whether a container may be collapsed when doing so would leave no usable sibling visible. Collapse state should be classified as persistent or transient deliberately.

### 5.6 Tab layout

The tab strip may support top, bottom, left, and right orientations. The design should determine how each orientation affects available content space, tab sizing, overflow, scrolling, and the placement of group controls.

Only the selected tab's content may need to be visible at a time, but the model and renderer must agree on how inactive tabs are retained and restored.

### 5.7 Embedded regions

An embed host may support top, bottom, left, and right regions. The geometry should be defined so that adjacent regions do not overlap unintentionally and their dimensions remain coherent when one region is resized.

If two adjacent edge regions leave a corner gap, the design should decide whether that gap is an intentional visual area, whether it can receive input, and how it relates to the host's underlying content.

The design should also establish whether embedded regions reserve space for the host header or may extend behind it.

### 5.8 Floating docks

Floating docks are intended to have a position and size within their host. The design should define whether they may move outside the host's bounds, whether clipping is allowed, how they stack relative to embedded regions, and how their resize handles work.

Movement and resizing should have clear minimum bounds and predictable behaviour near host edges. Whether floating docks can themselves contain arbitrary dock trees needs an explicit decision.

### 5.9 Visual stacking

The system will need a consistent layering policy for underlying content, embedded regions, floating docks, headers, resize handles, drop overlays, and user feedback. Exact layer values should be implementation details; the conceptual ordering and hit-testing priorities should be documented.

---

## 6. Parenting and docking behaviour

### 6.1 General operation

A docking operation changes the placement of a source dock relative to a target. It may move the source into an existing group, create a new group around the target, replace a target's position with a new group, or exchange positions.

The selected behaviour should define its preconditions, result, and failure cases. The source should remain intact unless the operation explicitly states otherwise.

### 6.2 Extensible behaviour registry

A registry is a candidate mechanism for allowing docking behaviours to be added or replaced. Each behaviour would receive enough context to identify the current layout, source, target, and intended drop region, then report whether it made a valid change.

The public contract must establish how behaviours are registered, how duplicate names are handled, how unknown behaviours are treated, and how failures are represented. A fallback to a default behaviour should not be assumed until deliberately selected.

### 6.3 Drop-region concept

A target may expose five conceptual regions: top, bottom, left, right, and centre. These regions communicate different results rather than merely indicating a pixel location.

The visual proportions of the regions, the treatment of corners, accessibility, and whether every dock category exposes all regions should be settled through interaction design and testing.

### 6.4 Candidate behaviour: split or insert

A split operation is intended to place the source beside the target along a chosen direction. If the target already belongs to a compatible container, the design may insert the source into that container; otherwise, it may create a new directional container around the source and target.

The design should determine ordering, size transfer, behaviour for targets in tabs or embedded regions, and whether a centre drop should have a separate meaning such as swapping.

### 6.5 Candidate behaviour: tab grouping

A tab-grouping operation is intended to make the source and target members of the same tab group. If the target already belongs to a suitable group, the source may join it; otherwise, a new group may be created around the two docks.

The design must specify tab order, which tab becomes active, how the strip orientation is chosen, and how groups are handled when a dock is dropped onto its own group.

### 6.6 Candidate behaviour: embedding

An embedding operation is intended to place the source in a selected edge region or as a floating dock inside a host. If an edge region is already occupied, the design must decide whether the existing dock is replaced, moved to a floating position, or handled another way.

The operation should also define what happens when the target is already a host, when it is not an eligible host, or when it belongs to a dock category that cannot safely become a host.

### 6.7 Preconditions and atomicity

Potential safeguards include rejecting a move onto the source itself, rejecting a move into the source's descendants, validating the requested region, and verifying that both source and target are still present.

A key design goal is that an unsuccessful operation should not leave a partially modified layout. Whether this is achieved through pre-validation, a reversible change plan, immutable transformations, or another method remains to be decided.

### 6.8 Behaviour compatibility

The system should document which source and target categories each docking behaviour supports. Unsupported combinations should have a deliberate outcome, such as a clear rejection or a defined alternative, rather than silently corrupting the layout.

---

## 7. Drag-and-drop interaction

### 7.1 Drag sources

Potential drag handles include panel headers, container headers, a tab-group handle, an individual tab grip, and floating-dock headers. The design should distinguish moving an entire group from moving one member of that group.

It must also decide how users discover hidden handles, how text selection and ordinary clicks are protected, and what input methods are supported beyond mouse-based dragging.

### 7.2 Interaction lifecycle

A typical drag interaction may include:

1. Identify the source dock and begin a drag.
2. Record the relevant transient interaction state.
3. Reveal eligible drop targets and their available regions.
4. Highlight the region currently under the pointer.
5. Update the selected docking behaviour when the user changes the applicable modifier or mode control.
6. Validate and apply the drop.
7. Refresh the rendered layout if the operation succeeds.
8. Clear transient state and restore the interface after success, cancellation, or failure.

The exact event sequence, browser APIs, and cleanup strategy are implementation decisions.

### 7.3 Drop overlays

Overlays may display eligible targets and their regions while a drag is active. The design should ensure that the source and its descendants cannot become invalid targets and that nested docks remain reachable when they are intended to be targets.

Overlay stacking and pointer hit-testing require careful validation, especially when targets are nested or embedded inside another dock.

### 7.4 Switching behaviours during a drag

The interaction may allow the user to change docking behaviour while dragging, for example through keyboard modifiers or an explicit mode control. The design should decide which input method is primary, how current mode changes are communicated, and which mode takes effect if input changes at the moment of drop.

Modifier handling should remain understandable and should have a clear reset policy when the window loses focus or the drag is cancelled.

### 7.5 Cancellation and failure

The system should have a defined outcome for drops outside valid targets, invalid operations, lost pointer state, window focus changes, and browser drag cancellation. Cleanup should not depend solely on the success path.

### 7.6 Input accessibility

The initial mouse interaction should not prevent later support for keyboard-driven docking, touch input, or assistive technologies. The design should decide whether those are in scope initially and how drop-region choices can be exposed without relying only on colour or pointer position.

---

## 8. Headers, controls, and commands

### 8.1 Dock controls

Potential controls include moving a dock, closing it, collapsing or restoring it, and selecting an active tab. The design should establish which controls apply to each dock category and whether closing a parent removes all descendants.

The root may intentionally have no ordinary dock header. This should be treated as a design choice rather than an accidental exception.

### 8.2 Context-sensitive visibility

To reduce clutter, some headers or group controls may remain hidden until the user requests them or points to a relevant area. The design should establish how controls are revealed, how long they remain visible, how nested docks behave, and how users discover that a hidden control exists.

The reveal mechanism should avoid interfering with dragging, tab selection, and embedded-dock interactions.

### 8.3 Commands

User actions should be represented through a clear command boundary. Candidate operations include moving or docking a dock, closing a dock, toggling collapse, changing the active tab, creating a dock in a demonstration environment, and showing or hiding drag overlays.

The command names, payloads, validation results, and error model are not yet established. The command layer should not need to manipulate visual elements directly.

### 8.4 Feedback

The interface may provide brief feedback about the selected docking behaviour, the region under the pointer, successful changes, and rejected operations. The design should avoid making transient notifications the only way to understand an operation's result.

### 8.5 Interaction conflicts

The design should explicitly handle conflicts among drag initiation, button clicks, tab activation, collapse controls, floating-window movement, and resizing. Each gesture should have a clear starting area and a predictable result.

---

## 9. Resizing and shared boundaries

### 9.1 General resizing model

Resizing should use a consistent reference point and a well-defined interpretation of pointer movement. A candidate approach is to capture the initial geometry at the start of a gesture and derive each update from the total movement since that point, rather than repeatedly applying small deltas.

The exact calculation method and state-update frequency remain implementation choices.

### 9.2 Splitter resizing

A splitter should adjust the sizes of the two adjacent regions it separates. The design must define how the system handles minimum sizes, what happens when one neighbour reaches a bound, and how the resulting sizes are stored.

The effect on other siblings in a container with more than two children should be documented and tested.

### 9.3 Shared junctions

Where multiple splitters meet, a shared junction control may allow the user to resize several connected boundaries together. This could make complex layouts easier to adjust, but it introduces geometry and consistency requirements.

The design should decide how junctions are detected, which boundaries are considered connected, how movement is constrained across all affected regions, and whether the operation is supported in the first version.

### 9.4 Embedded-region resizing

Each occupied edge region may expose a handle on its inner boundary. Resizing one region may affect the positions of adjacent regions, so the layout should be recalculated coherently.

The design must define minimum and maximum dimensions and determine how corner interactions should behave when two adjacent regions are resized together.

### 9.5 Floating-dock resizing and movement

Floating docks may expose edge and corner handles for resizing and a header area for movement. The design should determine whether resizing from a particular edge keeps the opposite edge fixed, how the dock is constrained by its host, and how header buttons avoid accidentally starting a move.

### 9.6 Persistence of resize results

Every resize gesture should have a defined relationship to the logical layout state. A change that is expected to survive a re-render or saved session should not exist only in visual styling. The exact commit timing and representation remain to be chosen.

---

## 10. Rendering, state, and persistence

### 10.1 Rendering model

The rendering layer should translate the logical layout into visible dock elements. A full render after structural changes is one possible approach; targeted updates are another. The chosen strategy should prioritize correctness first and then consider performance and interaction continuity.

If a full render is used, transient interface state such as an active reveal or drag indicator may need to be reapplied deliberately.

### 10.2 Structural and visual updates

The design should distinguish structural changes from visual-only interactions. Docking, closing, and changing tab membership alter the logical layout. Hover effects, overlays, and pointer feedback may not. Resizing sits between these categories because it affects visible geometry and may also alter persistent layout preferences.

The rules for committing each kind of change should be explicit.

### 10.3 Persistence format

The layout model should be suitable for serialization so that users or host applications can save and restore arrangements. The design must determine the schema, versioning policy, identity generation strategy, validation of imported layouts, and behaviour when a saved layout contains unsupported or outdated data.

A restoration API and migration policy should be considered before the format becomes public.

### 10.4 Identity management

Each dock should have a stable identity that is unique within the relevant layout. The system should define how identities are created, how imported layouts are checked for collisions, and whether identity generation is the responsibility of the core system or the host application.

### 10.5 Error recovery

The system should define how it handles malformed layouts, missing targets, unsupported dock categories, and failed operations. Possible strategies include rejecting the operation before mutation, applying changes through a reversible plan, or restoring a prior valid state. The selected approach should be documented and testable.

### 10.6 Undo and redo

Undo and redo are not assumed to exist in the initial design. However, the command and state model should be assessed for whether structural and resize operations could later be represented as reversible changes.

---

## 11. Extensibility and custom docking behaviours

### 11.1 Extension goal

The system should aim to let integrators introduce new docking behaviours without requiring changes throughout the core interaction and rendering layers.

### 11.2 Behaviour contract

A future extension contract should clarify:

- What information a behaviour receives.
- How it identifies source and target docks.
- How it interprets the drop region.
- What preconditions it must validate.
- How it reports success, rejection, and failure.
- Whether it may change more than one part of the layout.
- How it preserves layout invariants.
- How the system handles conflicting registrations.

These details are to be designed; no final plugin API is implied here.

### 11.3 Custom dock categories

Whether integrators may add entirely new dock categories is a separate question from adding new docking behaviours. New categories affect representation, rendering, validation, persistence, and compatibility, so the extension boundary should state whether they are supported.

### 11.4 Compatibility and fallback

Unknown or unavailable behaviours should have a predictable outcome. The system should not silently perform a different operation unless such fallback behaviour is an explicit part of the contract.

---

## 12. Candidate invariants and validation rules

The following are candidate invariants to review and formalize before implementation:

1. Every dock has a unique identity within a layout.
2. The layout contains one designated root.
3. A dock cannot become its own descendant.
4. Every dock occupies a valid slot for its parent category.
5. Moving a parent normally preserves its subtree.
6. Only eligible dock categories may contain children.
7. Only supported host categories may own embedded regions or floating docks.
8. A failed docking operation should not leave the source detached or the layout partially changed.
9. Structural cleanup should not remove the root.
10. Active-tab selection should remain valid after tab membership changes.
11. Stored sizing information should use a defined representation and be interpreted consistently.
12. Persisted state should not rely on transient DOM-only properties.
13. Every visible drag target should correspond to a valid operation or provide a clear rejection.
14. Closing a dock should have a defined effect on its descendants and parent.
15. Collapse behaviour should not make a dock impossible to restore.
16. Imported layouts should be validated before they are trusted or rendered.

These are not yet an enforced rule set. Each invariant needs an agreed scope, a test, and a defined response when it is violated.

---

## 13. Open questions and risks

### 13.1 Model and structure

- Is a tree the right representation for every placement type, or should floating and embedded placements use a more explicit slot model?
- Which dock categories can be nested inside tab groups or embedded regions?
- Should same-direction containers be preserved, merged, or handled according to an explicit normalization policy?
- Can a host remain an embed host after all embedded docks are removed?
- How should the root behave when it has no children or only one child?

### 13.2 Docking behaviour

- Should a centre drop swap two docks, activate a tab, or have mode-specific meanings?
- What should happen when a user drops a dock on its own tab group?
- Should an occupied embed region displace its existing dock, reject the operation, or offer a choice?
- How should unsupported source-target combinations be communicated?
- Should unknown docking modes be rejected or mapped to a default?

### 13.3 Sizing and geometry

- Should sizing be stored as relative weights, proportions, pixel preferences, or a combination?
- How should minimum sizes interact with limited available space?
- Should shared-junction resizing be included in the initial scope?
- Can floating docks exceed their host bounds, and should the host clip them?
- How should embedded regions and headers share the available space?

### 13.4 Interaction

- Which modifier keys, if any, should select docking modes?
- Should docking mode be changeable throughout a drag?
- How are hidden headers discovered and revealed?
- Which touch and keyboard interaction paths are required?
- How should nested drop overlays resolve overlapping hit areas?

### 13.5 State and extensibility

- Which state is persistent, transient, or derived?
- What is the serialized layout format and versioning policy?
- Are custom dock categories supported, or only custom behaviours?
- Is undo/redo required, planned, or explicitly out of scope?
- What level of atomicity is expected for custom operations?

These questions should be resolved through design review and focused prototypes rather than being treated as already answered.

---

## 14. Verification plan

Verification should be developed alongside the design. The initial plan should cover the following areas.

### 14.1 Structural tests

- Locate docks in every supported slot category.
- Move individual docks and parent groups.
- Verify that descendants remain intact after moving a group.
- Remove empty and single-child groups according to the approved policy.
- Test root-specific behaviour.
- Reject self-targeting and ancestor/descendant-invalid operations.
- Confirm that failed operations leave a valid layout.

### 14.2 Docking-behaviour tests

- Exercise each supported region for every built-in behaviour.
- Test compatible and incompatible source-target combinations.
- Test dropping into existing groups and creating new groups.
- Test occupied embed regions and floating placement.
- Test unknown, rejected, and failing custom behaviours.

### 14.3 Sizing tests

- Resize containers with two and several children.
- Verify minimum and maximum bounds.
- Test adjacent and shared boundaries.
- Resize embedded regions independently and together.
- Move and resize floating docks near host edges.
- Confirm which changes survive rendering and restoration.

### 14.4 Interaction tests

- Start and cancel drags from every supported handle.
- Confirm invalid targets are suppressed.
- Test nested overlays and overlapping hit areas.
- Change modes during a drag if that feature is retained.
- Test window blur, pointer exit, and unsuccessful drops.
- Confirm buttons, tabs, and resize handles do not accidentally trigger drags.

### 14.5 Persistence tests

- Save and restore representative layouts.
- Reject malformed layouts and duplicate identities.
- Verify active tabs, collapsed state, sizing, embedded regions, and floating positions.
- Test compatibility behaviour for older or unsupported schema versions.

### 14.6 Usability and accessibility tests

- Confirm that drop regions are visually understandable.
- Do not rely on colour alone to distinguish behaviours.
- Check keyboard access to essential controls.
- Test readability and hit-target usability in all supported tab orientations.
- Evaluate whether hidden headers and controls can be discovered without guesswork.

A test is not considered passed merely because the feature appears to work in one example. Acceptance criteria should be written for each behaviour before implementation is considered complete.

---

## 15. Design decisions to establish

Before treating this document as a formal implementation contract, the project should establish:

1. **Scope:** supported platforms and input methods for the first version.
2. **Terminology:** final names and meanings of dock categories, slots, and docking behaviours.
3. **Model:** logical representation, identity policy, and parent-child constraints.
4. **Mutation policy:** detachment, insertion, replacement, cleanup, and failure atomicity.
5. **Built-in behaviours:** exact semantics for splitting, tab grouping, embedding, and centre drops.
6. **Sizing:** representation of size preferences, minimums, maximums, and resize persistence.
7. **Interaction:** handles, overlays, mode selection, cancellation, and accessibility.
8. **Rendering:** full or targeted rendering, transient-state restoration, and hit-testing policy.
9. **Persistence:** serialized schema, versioning, validation, and restoration.
10. **Extensibility:** custom behaviour API, custom dock-category support, and error handling.
11. **Verification:** invariants, acceptance criteria, and test coverage.
12. **Future capabilities:** undo/redo, touch support, keyboard docking, and other features that may be deferred.

Until these decisions are made, this document should be treated as a **design proposal and planning baseline**, not as a description of implemented behaviour.
