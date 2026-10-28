Q: now give potential folder files structure of it..where AppShell structure tree
.
├── tests
│   ├── app_engine_tests/
│   │   ├── main.rs
│   │   └── ...
│   └── domain_foundation_tests/
│       ├── main.rs
│       └── ....
├── benches
│   └── domain_foundation_bench.rs
├── Cargo.lock
├── Cargo.toml
└── src
    ├── app_engine
    │   ├── capabilities
    │   │   ├── engine_ref.rs
    │   │   ├── mod.rs
    │   │   └── permission_checked_ref.rs
    │   ├── commands
    │   │   ├── command_context.rs
    │   │   ├── command_definition.rs
    │   │   ├── command_executor.rs
    │   │   ├── command_handler.rs
    │   │   ├── command_registry.rs
    │   │   ├── command_result.rs
    │   │   ├── command.rs
    │   │   └── mod.rs
    │   ├── communication
    │   │   ├── channel.rs
    │   │   ├── dispatcher.rs
    │   │   └── subscription.rs
    │   ├── concurrency
    │   │   ├── async_command_handler.rs
    │   │   ├── async_runtime.rs
    │   │   ├── async_task_handler.rs
    │   │   ├── mod.rs
    │   │   └── task_pool.rs
    │   ├── configuration
    │   │   ├── config.rs
    │   │   ├── config_schema.rs
    │   │   ├── config_store.rs
    │   │   ├── mod.rs
    │   │   └── preferences.rs
    │   ├── context
    │   │   ├── application_context.rs
    │   │   ├── context_manager.rs
    │   │   ├── context.rs
    │   │   ├── execution_context.rs
    │   │   ├── mod.rs
    │   │   └── session_context.rs
    │   ├── errors
    │   │   ├── async_error.rs
    │   │   ├── command_error.rs
    │   │   ├── communication_error.rs
    │   │   ├── configuration_error.rs
    │   │   ├── context_error.rs
    │   │   ├── engine_error.rs
    │   │   ├── history_error.rs
    │   │   ├── mod.rs
    │   │   ├── persistence_error.rs
    │   │   ├── plugin_error.rs
    │   │   ├── plugin_security_error.rs
    │   │   ├── resource_error.rs
    │   │   ├── scheduling_error.rs
    │   │   ├── serialization_error.rs
    │   │   ├── service_error.rs
    │   │   ├── state_error.rs
    │   │   └── task_error.rs
    │   ├── events
    │   │   ├── event_bus.rs
    │   │   ├── event_dispatcher.rs
    │   │   ├── event_handler.rs
    │   │   ├── event.rs
    │   │   └── mod.rs
    │   ├── execution
    │   │   ├── cancellation_token.rs
    │   │   ├── deadline.rs
    │   │   ├── mod.rs
    │   │   └── output_channel.rs
    │   ├── history
    │   │   ├── history_entry.rs
    │   │   ├── history_store.rs
    │   │   ├── mod.rs
    │   │   ├── redo.rs
    │   │   ├── replay.rs
    │   │   ├── transaction.rs
    │   │   └── undo.rs
    │   ├── lib.rs
    │   ├── lifecycle
    │   │   ├── lifecycle_manager.rs
    │   │   ├── lifecycle.rs
    │   │   ├── lifecycle_state.rs
    │   │   └── mod.rs
    │   ├── metadata
    │   │   ├── mod.rs
    │   │   ├── payload_schema.rs
    │   │   └── version.rs
    │   ├── mod.rs
    │   ├── persistence
    │   │   ├── file_backend.rs
    │   │   ├── memory_backend.rs
    │   │   ├── mod.rs
    │   │   └── storage_backend.rs
    │   ├── plugins
    │   │   ├── dynamic_loader.rs
    │   │   ├── mod.rs
    │   │   ├── plugin_context.rs
    │   │   ├── plugin_loader.rs
    │   │   ├── plugin_manager.rs
    │   │   ├── plugin_manifest.rs
    │   │   ├── plugin_permissions.rs
    │   │   ├── plugin_registry.rs
    │   │   └── plugin.rs
    │   ├── resources
    │   │   ├── mod.rs
    │   │   ├── resource_cache.rs
    │   │   ├── resource_dependency.rs
    │   │   ├── resource_handle.rs
    │   │   ├── resource_loader.rs
    │   │   ├── resource_manager.rs
    │   │   └── resource.rs
    │   ├── runtime
    │   │   ├── bootstrap.rs
    │   │   ├── mod.rs
    │   │   ├── runtime.rs
    │   │   └── runtime_state.rs
    │   ├── scheduling
    │   │   ├── job.rs
    │   │   ├── mod.rs
    │   │   ├── queue.rs
    │   │   ├── retry_policy.rs
    │   │   ├── scheduler.rs
    │   │   ├── schedule.rs
    │   │   └── trigger.rs
    │   ├── serialization
    │   │   ├── mod.rs
    │   │   ├── serialization_registry.rs
    │   │   └── serializer.rs
    │   ├── services
    │   │   ├── mod.rs
    │   │   ├── service_manager.rs
    │   │   ├── service_registry.rs
    │   │   └── service.rs
    │   ├── signals
    │   │   ├── mod.rs
    │   │   ├── signal_bus.rs
    │   │   ├── signal.rs
    │   │   └── subscription.rs
    │   ├── state
    │   │   ├── mod.rs
    │   │   ├── state_manager.rs
    │   │   ├── state.rs
    │   │   ├── state_store.rs
    │   │   └── state_transition.rs
    │   ├── tasks
    │   │   ├── mod.rs
    │   │   ├── task_context.rs
    │   │   ├── task_definition.rs
    │   │   ├── task_executor.rs
    │   │   ├── task_handler.rs
    │   │   ├── task_manager.rs
    │   │   ├── task_result.rs
    │   │   ├── task.rs
    │   │   └── task_state.rs
    │   └── testing
    │       ├── assertions.rs
    │       ├── mocks.rs
    │       ├── mod.rs
    │       └── test_engine.rs
    ├── domain_engine
    │   ├── mod.rs
    │   └── operations
    │       ├── canvas_command_handler.rs
    │       ├── canvas_operations.rs
    │       ├── layers_command_handler.rs
    │       ├── mod.rs
    │       └── timeline_command_handler.rs
    ├── domain_foundation
    │   ├── environments
    │   │   ├── canvas_2d
    │   │   │   ├── canvas.rs
    │   │   │   ├── error.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── object.rs
    │   │   │   ├── operations.rs
    │   │   │   └── transform.rs
    │   │   ├── layers
    │   │   │   ├── error.rs
    │   │   │   ├── hierarchy.rs
    │   │   │   ├── layer.rs
    │   │   │   ├── mod.rs
    │   │   │   └── operations.rs
    │   │   ├── mod.rs
    │   │   ├── node_graph
    │   │   │   ├── edge.rs
    │   │   │   ├── error.rs
    │   │   │   ├── graph.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── node.rs
    │   │   │   ├── operations.rs
    │   │   │   └── port.rs
    │   │   ├── table
    │   │   │   ├── cell.rs
    │   │   │   ├── column.rs
    │   │   │   ├── error.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── operations.rs
    │   │   │   ├── row.rs
    │   │   │   └── table.rs
    │   │   ├── text_editor
    │   │   │   ├── document.rs
    │   │   │   ├── error.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── operations.rs
    │   │   │   ├── position.rs
    │   │   │   └── range.rs
    │   │   ├── timeline
    │   │   │   ├── clip.rs
    │   │   │   ├── error.rs
    │   │   │   ├── keyframe.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── operations.rs
    │   │   │   ├── timeline.rs
    │   │   │   └── track.rs
    │   │   └── viewport_3d
    │   │       ├── error.rs
    │   │       ├── mod.rs
    │   │       ├── object.rs
    │   │       ├── operations.rs
    │   │       ├── scene.rs
    │   │       └── transform.rs
    │   ├── mod.rs
    │   ├── relations
    │   │   ├── definition.rs
    │   │   ├── dependency.rs
    │   │   ├── endpoint.rs
    │   │   ├── error.rs
    │   │   ├── evaluation.rs
    │   │   ├── invalidation.rs
    │   │   ├── mod.rs
    │   │   ├── propagation.rs
    │   │   ├── registry.rs
    │   │   └── relation.rs
    │   ├── testing
    │   │   ├── fixtures.rs
    │   │   └── mod.rs
    │   └── utilities
    │       ├── batch
    │       │   ├── mod.rs
    │       │   └── operation.rs
    │       ├── filtering
    │       │   ├── criteria.rs
    │       │   ├── filter.rs
    │       │   └── mod.rs
    │       ├── grouping
    │       │   ├── criteria.rs
    │       │   └── mod.rs
    │       ├── mod.rs
    │       ├── querying
    │       │   ├── evaluator.rs
    │       │   ├── mod.rs
    │       │   └── query.rs
    │       ├── sorting
    │       │   ├── criteria.rs
    │       │   └── mod.rs
    │       ├── tagging
    │       │   ├── mod.rs
    │       │   ├── tag.rs
    │       │   └── tag_set.rs
    │       └── validation
    │           ├── mod.rs
    │           ├── result.rs
    │           └── rule.rs
    ├── lib.rs
    └── main.rs

48 directories, 220 files



Based on your current AppShell structure and the proposed Docking Environment PRD, I would add `docking` as a new built-in environment under `domain_foundation/environments`, with its own model, structural operations, sizing rules, validation, and errors. Integration with `domain_engine` would follow the existing environment-operation pattern.

This is a proposed structure, not a claim that these files or APIs already exist.

## 1. Proposed folder and file structure

```

src/
├── domain_engine/
│   ├── mod.rs
│   └── operations/
│       ├── canvas_command_handler.rs
│       ├── canvas_operations.rs
│       ├── docking_command_handler.rs       # New
│       ├── docking_operations.rs            # New
│       ├── layers_command_handler.rs
│       ├── mod.rs
│       └── timeline_command_handler.rs
│
└── domain_foundation/
    ├── environments/
    │   ├── docking/                         # New environment
    │   │   ├── mod.rs
    │   │   ├── model.rs                     # Main docking state and root
    │   │   ├── panel.rs                     # Dockable panel definitions
    │   │   ├── container.rs                 # Split and structural containers
    │   │   ├── tab_group.rs                 # Tab membership and active tab
    │   │   ├── embed_host.rs                # Embedded host relationships
    │   │   ├── slot.rs                      # Child placement and host slots
    │   │   ├── floating.rs                  # Floating dock roots and state
    │   │   ├── sizing.rs                    # Relative sizes and constraints
    │   │   ├── operations.rs                # Public domain operations
    │   │   ├── normalization.rs             # Structural simplification
    │   │   ├── validation.rs                # Structural invariants
    │   │   └── error.rs                     # Docking-specific errors
    │   │
    │   ├── canvas_2d/
    │   ├── layers/
    │   ├── node_graph/
    │   ├── table/
    │   ├── text_editor/
    │   ├── timeline/
    │   ├── viewport_3d/
    │   └── mod.rs
    │
    ├── relations/
    ├── testing/
    ├── utilities/
    └── mod.rs

```

The existing `domain_foundation/environments/mod.rs` would declare the new environment module, and `domain_foundation/mod.rs` would expose it according to the crate's current visibility conventions.

## 2. Responsibility of each file

| File               | Responsibility                                                                                      |
| ------------------ | --------------------------------------------------------------------------------------------------- |
| `model.rs`         | Owns the overall docking state, root structure, and references to dockable elements.                |
| `panel.rs`         | Defines a dockable panel's identity and domain-level properties. It does not render the panel's UI. |
| `container.rs`     | Represents structural containers, including horizontal and vertical splits.                         |
| `tab_group.rs`     | Represents tab membership, ordering, and the active tab.                                            |
| `embed_host.rs`    | Defines host relationships and rules for embedded docking structures.                               |
| `slot.rs`          | Defines where an element can be placed, including structural and host-related placement.            |
| `floating.rs`      | Represents floating docking roots and their logical state, without managing OS windows.             |
| `sizing.rs`        | Defines relative sizing, constraints, resizing, and collapse-related domain rules.                  |
| `operations.rs`    | Provides the environment's public operation definitions and coordinates structural changes.         |
| `normalization.rs` | Simplifies valid structures after operations, such as removing unnecessary containers.              |
| `validation.rs`    | Checks invariants such as valid parent-child relationships, unique membership, and valid roots.     |
| `error.rs`         | Defines failures such as invalid placement, missing elements, and rejected structural changes.      |
| `tests/`           | Tests the environment's invariants and operation behavior independently of rendering.               |

### Why separate these responsibilities?

Docking is more than a tree of panels. It combines several related but distinct concepts:

- Structure: where panels and containers belong.
- Grouping: which panels share a tab group.
- Hosting: how embedded docking structures relate to their hosts.
- Sizing: how space is allocated and adjusted.
- Normalization: how the structure changes after moves or removals.
- Validation: whether the resulting state is legal.

Separating these concepts makes the environment easier to implement and test without turning `operations.rs` into a very large file.

## 3. How the main operations would fit together

The following is the proposed dependency flow, not an established AppShell API.

\#chatgpt-mermaid-\_r_2hf\_{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";font-size:16px;fill:rgb(237, 237, 237);}@keyframes edge-animation-frame{from{stroke-dashoffset:0;}}@keyframes dash{to{stroke-dashoffset:0;}}#chatgpt-mermaid-\_r_2hf\_ .edge-animation-slow{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 50s linear infinite;stroke-linecap:round;}#chatgpt-mermaid-\_r_2hf\_ .edge-animation-fast{stroke-dasharray:9,5!important;stroke-dashoffset:900;animation:dash 20s linear infinite;stroke-linecap:round;}#chatgpt-mermaid-\_r_2hf\_ .error-icon{fill:rgb(48, 48, 48);}#chatgpt-mermaid-\_r_2hf\_ .error-text{fill:rgb(237, 237, 237);stroke:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ .edge-thickness-normal{stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .edge-thickness-thick{stroke-width:3.5px;}#chatgpt-mermaid-\_r_2hf\_ .edge-pattern-solid{stroke-dasharray:0;}#chatgpt-mermaid-\_r_2hf\_ .edge-thickness-invisible{stroke-width:0;fill:none;}#chatgpt-mermaid-\_r_2hf\_ .edge-pattern-dashed{stroke-dasharray:3;}#chatgpt-mermaid-\_r_2hf\_ .edge-pattern-dotted{stroke-dasharray:2;}#chatgpt-mermaid-\_r_2hf\_ .marker{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2hf\_ .marker.cross{stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2hf\_ svg{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";font-size:16px;}#chatgpt-mermaid-\_r_2hf\_ p{margin:0;}#chatgpt-mermaid-\_r_2hf\_ .label{font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ .cluster-label text{fill:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ .cluster-label span{color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ .cluster-label span p{background-color:transparent;}#chatgpt-mermaid-\_r_2hf\_ .label text,#chatgpt-mermaid-\_r_2hf\_ span{fill:rgb(237, 237, 237);color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ .node rect,#chatgpt-mermaid-\_r_2hf\_ .node circle,#chatgpt-mermaid-\_r_2hf\_ .node ellipse,#chatgpt-mermaid-\_r_2hf\_ .node polygon,#chatgpt-mermaid-\_r_2hf\_ .node path{fill:rgb(52, 24, 11);stroke:rgb(172, 79, 35);stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .rough-node .label text,#chatgpt-mermaid-\_r_2hf\_ .node .label text,#chatgpt-mermaid-\_r_2hf\_ .image-shape .label,#chatgpt-mermaid-\_r_2hf\_ .icon-shape .label{text-anchor:middle;}#chatgpt-mermaid-\_r_2hf\_ .node .katex path{fill:#000;stroke:#000;stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .rough-node .label,#chatgpt-mermaid-\_r_2hf\_ .node .label,#chatgpt-mermaid-\_r_2hf\_ .image-shape .label,#chatgpt-mermaid-\_r_2hf\_ .icon-shape .label{text-align:center;}#chatgpt-mermaid-\_r_2hf\_ .node.clickable{cursor:pointer;}#chatgpt-mermaid-\_r_2hf\_ .root .anchor path{fill:rgb(175, 175, 175)!important;stroke-width:0;stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2hf\_ .arrowheadPath{fill:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2hf\_ .edgePath .path{stroke:rgb(175, 175, 175);stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .flowchart-link{stroke:rgb(175, 175, 175);fill:none;}#chatgpt-mermaid-\_r_2hf\_ .edgeLabel{background-color:rgb(0, 0, 0);text-align:center;}#chatgpt-mermaid-\_r_2hf\_ .edgeLabel p{background-color:rgb(0, 0, 0);}#chatgpt-mermaid-\_r_2hf\_ .edgeLabel rect{opacity:0.5;background-color:rgb(0, 0, 0);fill:rgb(0, 0, 0);}#chatgpt-mermaid-\_r_2hf\_ .labelBkg{background-color:rgba(0, 0, 0, 0.5);}#chatgpt-mermaid-\_r_2hf\_ .cluster rect{fill:rgb(48, 48, 48);stroke:rgba(255, 255, 255, 0.15);stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .cluster text{fill:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ .cluster span{color:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ div.mermaidTooltip{position:absolute;text-align:center;max-width:200px;padding:2px;font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";font-size:12px;background:rgb(48, 48, 48);border:1px solid rgba(255, 255, 255, 0.15);border-radius:2px;pointer-events:none;z-index:100;}#chatgpt-mermaid-\_r_2hf\_ .flowchartTitleText{text-anchor:middle;font-size:18px;fill:rgb(237, 237, 237);}#chatgpt-mermaid-\_r_2hf\_ rect.text{fill:none;stroke-width:0;}#chatgpt-mermaid-\_r_2hf\_ .icon-shape,#chatgpt-mermaid-\_r_2hf\_ .image-shape{background-color:rgb(0, 0, 0);text-align:center;}#chatgpt-mermaid-\_r_2hf\_ .icon-shape p,#chatgpt-mermaid-\_r_2hf\_ .image-shape p{background-color:rgb(0, 0, 0);padding:2px;}#chatgpt-mermaid-\_r_2hf\_ .icon-shape .label rect,#chatgpt-mermaid-\_r_2hf\_ .image-shape .label rect{opacity:0.5;background-color:rgb(0, 0, 0);fill:rgb(0, 0, 0);}#chatgpt-mermaid-\_r_2hf\_ .label-icon{display:inline-block;height:1em;overflow:visible;vertical-align:-0.125em;}#chatgpt-mermaid-\_r_2hf\_ .node .label-icon path{fill:currentColor;stroke:revert;stroke-width:revert;}#chatgpt-mermaid-\_r_2hf\_ .node .neo-node{stroke:rgb(172, 79, 35);}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node rect,#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].cluster rect,#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node polygon{stroke:url(#chatgpt-mermaid-\_r_2hf\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].swimlane.cluster rect{filter:none;}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node path{stroke:url(#chatgpt-mermaid-\_r_2hf\_-gradient);stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node .outer-path{filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node .neo-line path{stroke:rgb(172, 79, 35);filter:none;}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node circle{stroke:url(#chatgpt-mermaid-\_r_2hf\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].node circle .state-start{fill:#000000;}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].icon-shape .icon{fill:url(#chatgpt-mermaid-\_r_2hf\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2hf\_ [data-look="neo"].icon-shape .icon-neo path{stroke:url(#chatgpt-mermaid-\_r_2hf\_-gradient);filter:drop-shadow( 1px 2px 2px rgba(185,185,185,1));}#chatgpt-mermaid-\_r_2hf\_ .node text{font-size:14px;font-weight:600;letter-spacing:normal;fill:rgb(255, 183, 144);}#chatgpt-mermaid-\_r_2hf\_ .edgeLabels text{font-size:13px;font-weight:600;letter-spacing:-0.08px;fill:rgb(255, 183, 144);}#chatgpt-mermaid-\_r_2hf\_ .node tspan[font-weight="normal"],#chatgpt-mermaid-\_r_2hf\_ .edgeLabels tspan[font-weight="normal"]{font-weight:600;}#chatgpt-mermaid-\_r_2hf\_ .edgeLabel .label rect{opacity:1;rx:13px;ry:13px;fill:rgb(40, 17, 5);stroke:rgb(92, 56, 31);stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .node rect,#chatgpt-mermaid-\_r_2hf\_ .node circle,#chatgpt-mermaid-\_r_2hf\_ .node ellipse,#chatgpt-mermaid-\_r_2hf\_ .node polygon,#chatgpt-mermaid-\_r_2hf\_ .node path{fill:rgb(74, 34, 6);stroke:rgba(255, 255, 255, 0.1);stroke-width:1px;}#chatgpt-mermaid-\_r_2hf\_ .node rect{rx:16px;ry:16px;}#chatgpt-mermaid-\_r_2hf\_ .node.mermaid-decision .label-container{fill:rgb(40, 17, 5);stroke:rgb(92, 56, 31);stroke-dasharray:2,2;}#chatgpt-mermaid-\_r_2hf\_ .edgePaths .flowchart-link{stroke:rgb(175, 175, 175);stroke-width:1px;stroke-linecap:round;stroke-linejoin:round;}#chatgpt-mermaid-\_r_2hf\_ .marker{fill:rgb(175, 175, 175);stroke:rgb(175, 175, 175);}#chatgpt-mermaid-\_r_2hf\_ :root{--mermaid-font-family:-apple-system-body,ui-sans-serif,-apple-system,system-ui,"Segoe UI",Helvetica,"Apple Color Emoji",Arial,sans-serif,"Segoe UI Emoji","Segoe UI Symbol";}DomainEngineDocking command handlerDocking operationsDocking modelStructural containers and slotsSizing rulesNormalizationValidationCommit valid stateReject invalid operation

The intended flow is:

1. The Domain Engine receives a docking command.
2. The docking operation applies the requested structural change.
3. The resulting structure is normalized where necessary.
4. Validation checks the result.
5. A valid result is committed; an invalid result leaves the previous state unchanged.

The exact integration with AppShell's existing command, history, serialization, and state mechanisms must follow the implementations already present in the repository.

## 4. Which files should be created first?

I would implement the environment in these phases rather than creating every file at once.

## 1

Core model and invariants

`model.rs`, `panel.rs`, `container.rs`, `slot.rs`, `error.rs`, `validation.rs`

Establish the legal structure before adding complex mutations.

## 2

Basic docking operations

`operations.rs`, `normalization.rs`

Create, remove, split, move, group into tabs, and normalize the structure.

## 3

Sizing and advanced relationships

`sizing.rs`, `tab_group.rs`, `embed_host.rs`, `floating.rs`

Add resizing, tab activation, embedded hosts, and floating structures.

## 4

AppShell integration

`docking_command_handler.rs`, `docking_operations.rs`, and the necessary module declarations.

Connect the environment to existing Domain Engine conventions, then integrate history and persistence through established mechanisms.

## 5. Important design boundaries

- No UI files in this environment. Rendering, drag gestures, pointer hit-testing, drop overlays, and visual feedback belong to the frontend.
- No separate undo/redo subsystem. Reuse AppShell's established history and transaction mechanisms.
- No unnecessary generic framework. Keep docking-specific concepts in this environment unless another existing environment genuinely needs them.
- Keep structural mutations atomic. A failed move or split must not leave a partially modified layout.
- Treat file boundaries as provisional. If the existing environments use different patterns, follow those patterns instead of forcing this proposed organization onto the project.

One important refinement: I would not create separate `row_container.rs` and `column_container.rs` files by default. A shared split-container model with an orientation property is usually simpler, unless AppShell's design or implementation conventions give rows and columns genuinely different behavior.

Likewise, `tab_group.rs` should remain separate only if tab groups have their own meaningful state or invariants. The goal is a clear domain model, not the maximum number of files.