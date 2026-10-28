I have now opened the online documentation and found the dedicated guide: [Creating a New Environment](https://unique-digital-resources.github.io/AppShell-Documentation/domain_docs/guides/01-creating-environment/?utm_source=chatgpt.com). It explicitly walks through creating a new environment, from its module structure and data types to operations, tests, and Domain Engine integration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://unique-digital-resources.github.io\&sz=32)

AppShell



## The documented procedure

The guide uses a hypothetical `Measurement` environment as its example. Follow these steps to create an equivalent reusable environment.

1\. Create the module structure

Create a directory under `src/domain_foundation/environments/` containing `mod.rs`, a model file, supporting data types, `error.rs`, and `operations.rs`. Register the new module in `environments/mod.rs`.

2\. Define the data model and types

Define the environment's core state, stable IDs, associated data structures, and domain-specific errors. Expose useful methods for accessing and manipulating that state.

3\. Define operations and outcomes

Create an operation enum representing supported actions and an outcome enum representing their results. Implement `apply()` to execute each operation against the environment and return either an outcome or a domain error.

4\. Export the public API

Use `mod.rs` to expose the environment, its model types, errors, operations, and outcomes so consumers can use the environment through a stable module interface.

5\. Add tests and serialization support

Test model behavior, valid operations, invalid operations, and expected outcomes. The guide also covers standard Rust traits and optional Serde support.

6\. Integrate with `DomainEngine`

Add the environment to the engine, initialize it, expose a getter, add an `execute_*` method, integrate its error type, and publish the appropriate history entries, events, and signals. A command handler is optional.

Source: [Creating a New Environment](https://unique-digital-resources.github.io/AppShell-Documentation/domain_docs/guides/01-creating-environment/?utm_source=chatgpt.com) and [Integrating with DomainEngine](https://unique-digital-resources.github.io/AppShell-Documentation/domain_docs/guides/09-domain-engine-integration/?utm_source=chatgpt.com).&#x20;

[image](https://www.google.com/s2/favicons?domain=https://unique-digital-resources.github.io\&sz=32)

AppShell

+1



## One important distinction

The guide describes adding an environment to the existing `DomainEngine` by explicitly adding a field, getter, and execution method. It does not describe a generic mechanism for registering arbitrary third-party Foundations dynamically.

So, if by “new reusable foundation” you mean a new built-in environment, the guide gives you the procedure above. If you mean an independently packaged Foundation that applications can install as a plugin without modifying AppShell's `DomainEngine`, that is a different architectural requirement.

---

q: yes exactly "new built-in environment", now i want to establish a new one for headless ui docking system  AppShell's domain foundation  based on given Docking_System_Proposed_Design_Specification.md & DockingUi_keypoints.md and the documentaion

a : The right approach is to establish Docking System as a new built-in environment inside AppShell’s Domain Foundation, following the existing environment-creation and Domain Engine integration patterns—not as a standalone UI widget and not as a browser-specific implementation.

Your proposed specification already gives us the key architectural boundary: the docking environment owns the logical arrangement of docks and the rules for changing that arrangement, while the frontend owns rendering, pointer interactions, hit-testing, and visual feedback. The new environment should preserve that separation.&#x20;

Docking_System_Proposed_Design_Specification.md



## 1. What we are establishing

AppShell — Domain Engine

Hosts and exposes the built-in environment

## Docking Environment

A headless model and operation system

Layout model

Docks, groups, slots, hierarchy

Structural operations

Move, split, group, embed, remove

Layout constraints

Sizing, bounds, collapse, resize state

Validation

Invariants, invalid moves, safe changes

Frontend adapter / application UI

Renders panels, measures geometry, handles dragging, shows drop regions, and sends intentions to the environment.

The environment should not need a DOM, CSS, browser pointer events, or a particular UI framework to understand or validate a docking operation. That is consistent with the separation proposed in your specification.&#x20;

Docking_System_Proposed_Design_Specification.md



## 2. How it should fit AppShell

AppShell describes Domain Foundation environments as headless, reusable domain models composed by the Domain Engine. Its existing Canvas2D integration demonstrates the pattern of an operation type, an `execute_canvas(...)` method, access through `canvas()`, and undoable changes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



The Docking Environment should follow that pattern, adapting the model to hierarchical workspace layout.

| AppShell convention     | Proposed docking equivalent                                                   |
| ----------------------- | ----------------------------------------------------------------------------- |
| Environment module      | `docking`                                                                     |
| Environment state/model | A logical dock layout                                                         |
| Operation enum          | `DockingOperation`                                                            |
| Operation result        | The created dock ID, updated layout result, or another defined outcome        |
| Execution method        | `execute_docking(...)`                                                        |
| Read-only access        | `docking()`                                                                   |
| Error handling          | Docking-specific errors integrated with Domain Engine errors                  |
| Undo information        | Sufficient information to reverse structural and sizing changes               |
| Validation              | Docking-specific invariants and slot compatibility                            |
| Optional serialization  | Saved layout representation compatible with AppShell's serialization approach |

These are proposed API names, not existing AppShell APIs.

## 3. Proposed module structure

Following the environment-creation approach, I would start with this structure and adjust the filenames to match the actual conventions of the existing environment modules.

```

src/
└── domain_foundation/
    └── environments/
        ├── mod.rs
        └── docking/
            ├── mod.rs
            ├── model.rs
            ├── dock.rs
            ├── layout.rs
            ├── slot.rs
            ├── operation.rs
            ├── operations.rs
            ├── sizing.rs
            ├── validation.rs
            └── error.rs

```

The responsibilities would be:

`model.rs`, `dock.rs`, `layout.rs`

Define the layout's logical representation, dock identity and category, parent-child relationships, root, and supported placements.

`slot.rs`

Represent where a dock resides: an ordered child position, an edge region of a host, or a floating placement. Operations should work against these logical slots rather than visual coordinates.

`operation.rs`, `operations.rs`

Define user-intention operations and implement the actual structural transformations, including moving, splitting, tab grouping, embedding, and removal.

`sizing.rs`

Define size preferences, bounds, splitter changes, embedded-region dimensions, and floating-dock geometry as logical state.

`validation.rs`, `error.rs`

Enforce identity uniqueness, valid placement, parent-child rules, cycle prevention, active-tab validity, and safe failure behaviour.

This is a candidate organization, not a claim that these files are required by AppShell. Some responsibilities may belong together if that better matches the existing environment implementations.

## 4. Establish the core domain model first

The most important design decision is to define what the environment knows before defining how a frontend displays it.

Your specification proposes five main dock categories: panels, row containers, column containers, tab groups, and embed hosts.&#x20;

Docking_System_Proposed_Design_Specification.md



I would establish the following conceptual model:

Docking Layout

One root and a valid collection of dock identities and placements

Panel

An individual work area

Split container

Ordered children arranged horizontally or vertically

Tab group

Members plus active selection

Embed host

Underlying content, edge regions, and floating docks

A few rules should be established early:

- Stable identity: a dock keeps its identity when moved or reconstructed.
- Explicit placement: each dock occupies a valid location in the model.
- Intact movement: moving a group normally moves its entire subtree.
- No invalid cycles: a dock cannot become its own descendant.
- Atomic changes: failed operations must not leave the layout partially modified.
- Explicit normalization: removing a dock may remove redundant groups, but must preserve groups with meaningful semantics.

### A critical distinction: logical layout versus geometry

The environment can store relative size preferences, minimum dimensions, and floating-dock positions. However, it should not decide where a pointer lands or measure DOM rectangles.

For example:

- The frontend determines that the pointer is over the left drop region of a target.
- It submits a docking intention containing the source, target, and requested operation.
- The Docking Environment validates the request and updates the logical layout.
- The frontend renders the resulting layout and recalculates its visual geometry.

This lets the same environment potentially serve browser UIs, native desktop UIs, or other frontends.

## 5. Define operations around user intentions

I recommend organizing the initial operation contract around these categories.

| Operation category | Purpose                                                     |
| ------------------ | ----------------------------------------------------------- |
| Creation           | Create the initial root and supported dock categories       |
| Movement           | Move a dock or group to a valid destination slot            |
| Split / insert     | Place a dock beside another dock horizontally or vertically |
| Tab grouping       | Add a dock to an existing tab group or create a group       |
| Embedding          | Place a dock in a host's edge region or floating collection |
| Selection          | Change the active member of a tab group                     |
| Resizing           | Change relative sizing or a supported region's dimensions   |
| Collapse           | Change the collapse state where supported                   |
| Removal            | Close or remove a dock and normalize eligible parent groups |
| Validation         | Check whether the layout and requested operation are valid  |

These should be domain operations, not mouse-event handlers. The distinction also makes them easier to test without a graphical interface.

One important design choice remains open: should resizing, tab selection, and collapse all be represented as persistent layout mutations, or should some of them remain transient UI state? I would make that distinction explicit in the operation contract rather than assume all interaction state belongs in the environment.

## 6. Integrate it into `DomainEngine`

Once the standalone environment model and operations are established, integrate it using the same general pattern as the existing environments:

1. Register the module under `domain_foundation::environments`.
2. Add the docking state to `DomainEngine`.
3. Initialize a valid default docking layout.
4. Expose read access through a docking accessor.
5. Add an execution entry point for docking operations.
6. Integrate docking errors with the existing error model.
7. Ensure operations provide the undo information required by AppShell's history mechanism.
8. Add tests for execution, undo, invalid operations, and restoration.
9. Add serialization support in line with the project's existing feature conventions.

AppShell's public README documents undo/redo, transactions, and optional Serde support for Domain Foundation types, so these should be considered during the initial design rather than bolted on after the layout model becomes public.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 7. Implementation phases

I would establish the environment in the following order.

1. Phase 1 — Model and invariants

   Define dock categories, stable IDs, the root, parent-child relationships, slots, and structural validation. No frontend interaction yet.
2. Phase 2 — Core structural operations

   Implement creation, lookup, detach/reattach, split insertion, tab grouping, removal, cleanup, and safe failure behaviour.
3. Phase 3 — Sizing and host placement

   Establish relative sizing, minimum bounds, embedded edge regions, floating positions, collapse semantics, and resize operations.
4. Phase 4 — Domain Engine integration

   Add the environment state, accessor, execution method, error integration, and undoable operations.
5. Phase 5 — Persistence and verification

   Define serialization and restoration rules, malformed-layout handling, invariant tests, and undo/redo tests.
6. Phase 6 — Frontend adapter

   Implement drag-and-drop, drop overlays, pointer hit-testing, rendering, headers, splitters, and interaction feedback outside the headless environment.

## 8. What I would settle before writing implementation code

There are three architectural decisions I would resolve first:

1. Layout representation: should embedded and floating placements be explicit slot collections separate from ordinary child relationships, or be represented entirely within one tree?
2. Atomicity and undo: should each operation calculate and validate a complete change before committing it, and what exact data must it retain to reverse the change?
3. Sizing semantics: should split children store relative weights, normalized proportions, or another representation—and how should those values transfer when groups are created, removed, or moved?

My recommendation is to begin with Phase 1: the formal model and invariants, then derive the operation enum and module structure from those decisions. This is where your specification has the greatest complexity: the docking modes interact with nested groups, host promotion, cleanup, and sizing. Establishing those rules first will make the Rust implementation much more predictable.

One limitation of my verification: the public AppShell README confirms the general environment integration pattern, but I could not retrieve the detailed environment-creation guide itself in this search. The proposed names and file tree above therefore remain recommendations to align with the actual source conventions, not a verbatim reproduction of that guide.