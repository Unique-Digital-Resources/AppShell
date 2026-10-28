# Docking System — Main Design Key Points

The design describes a proposed docking system for arranging, moving, resizing, and managing application panels. Its goal is to establish the rules and mechanisms for a flexible docking environment, rather than assume that any particular implementation already exists.

## 1. Core concepts

[Web Components Dock Manager | Layout Controls | Infragistics](https://images.openai.com/static-rsc-4/oI8Po4yyqiR9a-_-v9g4_N9EuzjnFV8rz3-CfY6BsLL5OCmXCZpgJPr-ShJktie5_8M3vEehyo4RcdQRlrTqeMiDMGTHavBFLJ_KZ8Y9SiLqJdOOvbfhy1XW6JmcjHHFkFwUE8nT-z-2nLXlAAIcmDAOO_i9CpqqC_D9H1mDzlI?purpose=inline)

Dockable panels

Panels are the primary units users interact with. They may represent tools, editors, inspectors, or other application views.

[Can I save/restore my split-window layout in the Eclipse Java editor? - Stack Overflow](https://images.openai.com/static-rsc-4/W79m6nzRHrK8VF6OPPdfEAFi6IBLkEj_hqI143I8iRDoN1_qjn28dsLAc1X0ZrYi4Dq6l_Rs74O7KFOUJPgq1fIYou4l6FDTmfVWpzIBQYIrr1ZuFTrJ5szLV-UqGK8qLluHA6v-RIn8Cvq5Rr8jm6jQr8AhnPfhD9JaTmJRSKc?purpose=inline)

Dock layout

A structured arrangement determines where panels appear, how they relate to one another, and how available space is distributed.

[Year 2025 Posts - The Actipro Blog - WPF, Avalonia UI, and WinForms Development](https://images.openai.com/static-rsc-4/GBwRCGXWsuLx0Cv7xHu8IsXyhD59ObipY7ufICIo86oMCzU67NPrl-5LqacJmJ62whrLk3KffqhHNpwK6Qh1mEfEaszcVBQNQgsZ_JPhr3slFTPwkk_DyqoVquG70FnmDE6SjPSGSopY2co9BjLsOo_PMRYsN8Ikc0ndtBrLOtk?purpose=inline)

Docking relationships

Panels can participate in different arrangements, including side-by-side splits, tab groups, edge docking, and floating windows.

## 2. Main behavioural rules

- Docking changes structure: moving a panel can change the layout, not just its screen coordinates.
- Splitting: panels can be arranged horizontally or vertically, with space shared between them.
- Tab grouping: multiple panels can occupy the same group, with one active panel displayed at a time.
- Edge docking: dragging a panel toward a valid target or edge can propose a new docking relationship.
- Floating: a panel can be detached from the main dock layout and placed independently.
- Resizing: users can adjust the space allocated to panels or groups, subject to their size constraints.
- Dragging and dropping: the system needs to distinguish a panel being moved, a potential target, a valid drop location, and the final layout change.
- Panel controls: headers and related controls provide a place for actions such as activation, movement, closing, and other panel-specific operations.

These are intended design behaviours; their exact rules, priorities, and edge cases still need to be established.

## 3. The three main docking modes

These are the most important mechanisms in the design.

[전체 화면 구성](https://images.openai.com/static-rsc-4/fW8TPnhE7aMoE2oNjqJPYbaHeaFsgcOQifLRhf6zEAfwBx_5Qf8RKR2q3uJ-Y1LLQmz1a_e_P6Fn5QkvSCjOet3-4-bIL5LBhRZTciPbqx1BZp_vQ7ygBD3zAwEusjNK_B0RMaxGg1on78S9iuglMaMpgZlVKZGc8XS7uUiTp68?purpose=inline)

1\. Split / Insert

Places one dock beside another, horizontally or vertically. The system may create a new row or column around the two docks, or insert into an existing compatible container.

[MacでのVSCode入門 #初心者 - Qiita](https://images.openai.com/static-rsc-4/hqrw8U2CtrWZyCuM-5-Swcn2FI0bR3snPuwDbYRRU5hCh6p3I4rfuBPPlwWMbF16ntU_yrvTsc_LDreeK6_boeiTtKdAsUvNRKE2YSuufsWxDnPcTzl3tva3TXvq3PyUAEtW6XhpNv3d5MKdRm4GZtMjo8iolBNy0abV37u9Ipw?purpose=inline)

2\. Tab grouping

Combines multiple docks into one tab group. Only the active dock's content needs to be displayed, while the group retains its members.

[DockControl: Customizable Workspaces for WPF | ComponentOne](https://images.openai.com/static-rsc-4/9Go0F-SPKyLFWKIpFct73GHqeS2I-MFlh4e0RDKIDwEN2ySut7IPrZOYBPKD_Kd_vZEJJ8sx2S4YGqjOJSV6lT_ZSeuxKZowYUaKSaGTQBA7cAQEw65oOB5xn50P8-EF1DodiFg1JaJ9loG2U_s_Jzj8sBE4bTV4DsmkxFaqXkg?purpose=inline)

3\. Embedding

Turns an eligible dock into a host for additional docks positioned at its edges or as floating panels inside it. The host's original content remains underneath.

A key design decision is that the drop region and docking mode work together: the region indicates where the dock should go, while the mode determines what structural operation happens.

## 4. The layout model and structural rules

The layout is intended to be represented as a logical structure, potentially a tree, rather than as unrelated visual elements.

Important rules to establish:

- Stable identity: each dock should be identifiable even when its rendered interface is rebuilt.
- Move groups intact: moving a container should normally move its entire subtree, not scatter its children.
- Valid parent-child relationships: a dock cannot become its own descendant or occupy an unsupported slot.
- Structural cleanup: empty or redundant containers may be removed, while meaningful groups are preserved.
- Explicit placement slots: ordinary child positions, embedded edge regions, and floating positions must all be represented.
- Safe operations: a rejected docking operation should not leave the layout partially modified.

The design also considers promoting a dock into an embed host when necessary, and potentially restoring its previous category when no embedded or floating docks remain.

## 5. Resizing and geometry

The system is intended to support more than basic splitters.

- Splitters: resize adjacent panels or regions.
- Shared junctions: potentially adjust several connected boundaries together.
- Embedded-region handles: change the dimensions of edge-docked areas.
- Floating-panel handles: resize floating docks independently.
- Bounds: enforce appropriate minimum and potentially maximum dimensions.
- Size preservation: retain sizing preferences when panels move, groups change, or the interface is rendered again.

Relative sizing is a candidate approach for split containers. The exact sizing representation, constraint handling, and resize calculations still need to be defined.

## 6. Interaction design

The proposed interaction flow is:

Select a dock or group handle

Reveal eligible drop targets

Choose a region and docking mode

Validate and apply the layout change

Refresh the interface and clear drag state

Additional interaction principles include:

- Drop overlays make valid targets and regions visible.
- A group handle moves the group, while an individual tab grip moves one member.
- Dock headers and group controls may be revealed contextually to reduce workspace clutter.
- The interface needs defined cancellation, failure, focus-loss, and invalid-drop behaviour.
- Keyboard, touch, and assistive-technology support require explicit scope and design decisions.

## 7. Architecture and separation of responsibilities

The design proposes separating the system into conceptual layers:

| Layer                      | Responsibility                                             |
| -------------------------- | ---------------------------------------------------------- |
| Layout model               | Represents docks, hierarchy, ordering, and layout state    |
| Structural operations      | Finds, moves, replaces, and validates docks                |
| Docking behaviour registry | Selects and delegates to the appropriate docking operation |
| Command boundary           | Represents user intentions independently of presentation   |
| Interaction layer          | Handles dragging, input, modes, and feedback               |
| Rendering layer            | Turns the logical layout into a visible interface          |
| Geometry and resizing      | Handles boundaries, measurements, and size changes         |
| Persistence                | Saves, restores, and validates layouts                     |

The central architectural idea is to keep layout logic independent of the user interface. That would make structural behaviour easier to test and potentially allow different frontends to use the same core system.

## 8. Persistence and extensibility

Two other important goals are:

- Persistence: support saving and restoring layouts, with stable identities, schema versioning, validation, and a policy for incompatible saved data.
- Extensibility: allow new docking behaviours through a defined extension boundary, without rewriting the entire drag-and-drop system.

Undo/redo is a future possibility, not an assumed initial feature. Adding entirely new dock categories is also a separate decision from adding new docking behaviours.

## 9. What still needs to be decided

The specification deliberately leaves several matters open rather than presenting them as implemented:

- The exact layout representation and public API.
- Whether centre drops swap docks or have another meaning.
- How occupied embedded regions are handled.
- How sizing preferences transfer during moves and structural cleanup.
- Whether floating docks may contain arbitrary nested layouts.
- How persistence, restoration, and migration work.
- How docking behaviours are registered, rejected, and given fallbacks.
- Which accessibility and input methods are included initially.
- Whether shared-junction resizing and undo/redo belong in the first version.

My assessment: the strongest part of this design is the combination of a structured layout model, multiple docking behaviours, and a separation between interaction and structural operations. The most important design task is to formalize the invariants and the behaviour of every docking operation before implementation begins. That will prevent edge cases in nesting, resizing, and moving groups from undermining the system's flexibility.