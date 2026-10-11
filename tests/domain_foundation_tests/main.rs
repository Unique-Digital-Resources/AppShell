//! Phase 2–12 integration tests.
//!
//! Phase 12 adds acceptance, composition, and public API tests.

mod canvas_2d_test;
mod canvas_2d_operations_test;
mod relation_definition_test;
mod relation_registry_test;
mod relation_evaluation_test;
mod relation_propagation_test;
mod domain_operation_integration_test;
mod canvas_history_integration_test;
mod timeline_test;
mod timeline_operations_test;
mod layers_test;
mod layers_operations_test;
mod node_graph_test;
mod node_graph_operations_test;
mod node_graph_canvas_integration_test;
mod viewport_3d_test;
mod viewport_3d_operations_test;
mod table_test;
mod table_operations_test;
mod text_editor_test;
mod text_editor_operations_test;
mod filtering_test;
mod tagging_test;
mod querying_test;
mod sorting_test;
mod grouping_test;
mod validation_test;
mod batch_test;
mod domain_foundation_test;
mod domain_composition_test;
mod domain_foundation_acceptance_test;
mod concurrency_test;
mod proptest_test;
mod canvas_command_handler_test;
mod relation_value_source_test;
#[cfg(feature = "serde")]
mod serde_roundtrip_test;
mod app_engine_lifecycle_test;
mod stress_test;