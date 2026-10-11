//! # Relations
//!
//! Relations describe how changes in one object or property affect other
//! objects or properties. This is more specific than "relations connect
//! objects": a relation represents propagation, dependency, derivation or
//! invalidation semantics between endpoints.
//!
//! ```text
//! Object A.property X
//!       │  changes
//!       ▼
//!    Relation
//!       │
//!       ├──→ Object B.property Y
//!       ├──→ Object C.property Z
//!       └──→ Object D.property Q
//! ```
//!
//! ## Mechanism vs meaning
//!
//! The reusable Relation system provides the *mechanism*:
//!
//! - relation identity and definitions,
//! - source/target endpoints (object + property),
//! - dependency representation,
//! - registration and lookup,
//! - evaluation of applicable relations on property change,
//! - propagation and invalidation.
//!
//! The **Domain Engine** defines the *meaning*: which relations exist, what
//! property changes, what effect occurs, when it occurs, whether it
//! propagates, and how the result is calculated. The foundation must not
//! hardcode application-specific effects.
//!
//! ## Independence from environments
//!
//! Relations are reusable independently of any particular environment. An
//! environment may *use* relations (Layers using hierarchy relations,
//! NodeGraph using port/edge relations, Timeline using temporal dependency
//! relations), but the relation system has no knowledge of those
//! environments' concrete types. All addressing is via opaque
//! [`PropertyId`](endpoint::PropertyId) values.

pub mod definition;
pub mod dependency;
pub mod endpoint;
pub mod error;
pub mod evaluation;
pub mod invalidation;
pub mod propagation;
pub mod registry;
pub mod relation;

pub use definition::{
    EffectContext, EffectFn, EffectOutcome, PropagationMode, PropertyValue,
    RelationDefinition, RelationDefinitionBuilder, Trigger,
};
pub use endpoint::{Endpoint, ObjectId, PropertyId};
pub use error::RelationError;
pub use registry::{RelationRegistry, RelationRegistryStats};
pub use relation::{Relation, RelationId};