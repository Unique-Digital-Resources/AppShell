//! # NodeGraph Environment
//!
//! Headless graph editing model with spatial placement composed from
//! Canvas2D. Provides nodes, input/output ports, edges and connection
//! validation without imposing any UI or rendering.
//!
//! ## Composition with Canvas2D
//!
//! ```text
//!                NodeGraph
//!               /          \
//!              ▼            ▼
//!          Canvas2D     Graph connectivity
//!      (spatial state)  (nodes, ports, edges)
//! ```
//!
//! Each node has a [`NodeId`] (graph identity) and an
//! [`ObjectId`](crate::domain_foundation::environments::canvas_2d::ObjectId)
//! (spatial identity). Graph operations (connect, disconnect) don't touch
//! spatial state. Spatial operations (move, transform) don't touch graph
//! connectivity.
//!
//! ## What this environment owns
//!
//! - [`Node`] and [`Port`] identity, [`Edge`] connectivity.
//! - Input/output port direction and type-name matching.
//! - Connection validation (output→input, type match, no self-loop).
//! - Spatial placement via an internal [`Canvas`](crate::domain_foundation::environments::canvas_2d::Canvas).
//!
//! ## What this environment deliberately does *not* own
//!
//! - Type semantics (what "color" or "position" *means*).
//! - Execution/dataflow (actually pushing values through edges).
//! - Rendering of nodes, ports, or edges.

pub mod edge;
pub mod error;
pub mod graph;
pub mod node;
pub mod operations;
pub mod port;

pub use edge::{Edge, EdgeId};
pub use error::NodeGraphError;
pub use graph::NodeGraph;
pub use node::{Node, NodeId};
pub use operations::{NodeGraphOperation, NodeGraphOutcome};
pub use port::{Port, PortDirection, PortId};