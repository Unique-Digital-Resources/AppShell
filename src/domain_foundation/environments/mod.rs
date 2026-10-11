//! # Environments
//!
//! An **Environment** is a reusable, headless model for a particular kind of
//! domain organisation or editing context. It defines the semantics necessary
//! to work with objects within that kind of context.
//!
//! An Environment is *not* a UI component. It does not mean "a visual area on
//! the screen". It means: a headless domain model describing how objects can
//! be organised and manipulated in this particular kind of context.
//!
//! ## No universal abstraction
//!
//! The environments below do **not** share a common base trait. Each owns its
//! internal semantics. Forcing them into a hierarchy such as
//! `Environment → OrderedEnvironment → {Timeline, Layers, Table}` would be
//! an abstraction based on superficial UI similarity, not genuine reuse.
//!
//! ```text
//! Environment (category, not a trait)
//!    ├── Canvas2D        (Phase 2 — present)
//!    ├── Timeline        (Phase 5 — present)
//!    ├── Layers          (Phase 6)
//!    ├── NodeGraph       (Phase 7)
//!    ├── Viewport3D      (Phase 8)
//!    ├── Table           (Phase 9)
//!    └── TextEditor     (Phase 10)
//! ```
//!
//! ## Natural composition is allowed
//!
//! Environments may compose other environments when their semantics genuinely
//! require it. For example, `NodeGraph` will use `Canvas2D` for spatial
//! placement while owning its own graph connectivity semantics. Such
//! composition is between concrete environments, never through a forced
//! shared base.
//!
//! ## Operations vs execution
//!
//! An Environment defines *what an operation means* (e.g. "move object",
//! "insert clip", "delete range"). The generic `app_engine` provides the
//! execution infrastructure: commands, tasks, history, events, signals,
//! scheduling. Environments therefore never become mini App Engines.

pub mod canvas_2d;
pub mod timeline;
pub mod layers;
pub mod node_graph;
pub mod viewport_3d;
pub mod table;
pub mod text_editor;

// All environments implemented (Phases 2–10).
// No `Environment` trait is declared here. Intentional.
//
// No `Environment` trait is declared here. Intentional.