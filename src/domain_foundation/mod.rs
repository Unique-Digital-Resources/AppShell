//! # Domain Foundation
//!
//! Reusable, headless domain capabilities that sit between application-specific
//! `domain_engine` modules and the generic `app_engine` infrastructure.
//!
//! The foundation is organised into three independent categories. They are
//! *categories of capability*, not a hierarchy:
//!
//! - [`environments`] — reusable models for particular kinds of domain
//!   organisation or editing context (Canvas2D, Timeline, Layers, Table,
//!   NodeGraph, TextEditor, Viewport3D, Docking, ...).
//! - [`relations`] — reusable mechanisms describing how changes to one
//!   object or property affect other objects or properties.
//! - [`utilities`] — optional capabilities (filtering, tagging, querying,
//!   sorting, grouping, validation, batch operations) that a Domain Engine
//!   may choose to compose.
//!
//! ## Design principles
//!
//! - **Composition over artificial generalisation.** A concept becomes
//!   reusable only when it has a real, meaningful boundary. The foundation
//!   deliberately does *not* introduce a universal `Environment` trait or
//!   a shared `OrderedEnvironment` base. Environments that do not share
//!   genuine semantics do not share a base.
//! - **Headless by construction.** No UI, rendering, input, animation, or
//!   layout concerns live here. The foundation describes *meaning*; the
//!   frontend presents it.
//! - **Opt-in utilities.** Declaring the [`utilities`] module does not make
//!   filtering, tagging, or validation mandatory for any environment. A
//!   Domain Engine explicitly selects what it composes.
//! - **Dependency direction.** The arrow always points downward:
//!   `domain_engine → domain_foundation → app_engine`. The `app_engine`
//!   module must never depend on `domain_foundation` concepts.
//!
//! ## Current state
//!
//! Phase 1 establishes only the module boundaries, public API surface and
//! visibility rules. Concrete environments, the relation engine and optional
//! utilities are introduced in subsequent phases, each only when needed.

pub mod environments;
pub mod relations;
pub mod utilities;

/// Shared test support for domain systems.
///
/// Public so that integration tests under `tests/` can reuse fixtures. This
/// module contains no production behaviour.
pub mod testing;