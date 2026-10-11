//! `ResourceDependencyGraph` — tracks which resources depend on which.
//!
//! If resource A depends on resource B, then B must be loaded before A,
//! and B cannot be unloaded while A is still loaded.

use std::collections::{HashMap, HashSet};

/// Maps a resource ID to the set of resource IDs it depends on.
type Dependencies = HashMap<String, HashSet<String>>;

/// Maps a resource ID to the set of resource IDs that depend on it.
type Dependents = HashMap<String, HashSet<String>>;

/// Tracks dependency relationships between resources.
#[derive(Debug, Clone, Default)]
pub struct ResourceDependencyGraph {
    /// resource_id -> set of dependencies (what this resource needs)
    dependencies: Dependencies,
    /// resource_id -> set of dependents (what needs this resource)
    dependents: Dependents,
}

impl ResourceDependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Register that `resource_id` depends on `dependency_id`.
    pub fn register_dependency(
        &mut self,
        resource_id: &str,
        dependency_id: &str,
    ) {
        self.dependencies
            .entry(resource_id.to_string())
            .or_default()
            .insert(dependency_id.to_string());
        self.dependents
            .entry(dependency_id.to_string())
            .or_default()
            .insert(resource_id.to_string());
    }

    /// Get the dependencies of a resource (what it needs).
    pub fn dependencies_of(&self, resource_id: &str) -> Vec<String> {
        self.dependencies
            .get(resource_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the dependents of a resource (what needs it).
    pub fn dependents_of(&self, resource_id: &str) -> Vec<String> {
        self.dependents
            .get(resource_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a resource has any dependents (things that need it).
    pub fn has_dependents(&self, resource_id: &str) -> bool {
        self.dependents
            .get(resource_id)
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    /// Remove all dependency records for a resource.
    pub fn remove(&mut self, resource_id: &str) {
        // Remove from dependencies map
        if let Some(deps) = self.dependencies.remove(resource_id) {
            for dep in deps {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.remove(resource_id);
                }
            }
        }

        // Remove from dependents map
        if let Some(dependents) = self.dependents.remove(resource_id) {
            for dependent in dependents {
                if let Some(deps) = self.dependencies.get_mut(&dependent) {
                    deps.remove(resource_id);
                }
            }
        }
    }

    /// Get all resource IDs that are known in the dependency graph.
    pub fn all_resources(&self) -> Vec<String> {
        let mut all: HashSet<String> = HashSet::new();
        all.extend(self.dependencies.keys().cloned());
        all.extend(self.dependents.keys().cloned());
        all.into_iter().collect()
    }

    /// Resolve the load order for a set of resources (topological sort).
    /// Returns resources in dependency order (dependencies first).
    pub fn load_order(&self, resource_ids: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        fn visit(
            id: &str,
            graph: &ResourceDependencyGraph,
            visited: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) {
            if visited.contains(id) {
                return;
            }
            visited.insert(id.to_string());

            // Visit dependencies first
            for dep in graph.dependencies_of(id) {
                visit(&dep, graph, visited, result);
            }

            result.push(id.to_string());
        }

        for id in resource_ids {
            visit(id, self, &mut visited, &mut result);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_query() {
        let mut graph = ResourceDependencyGraph::new();
        graph.register_dependency("A", "B");
        assert_eq!(graph.dependencies_of("A"), vec!["B".to_string()]);
        assert_eq!(graph.dependents_of("B"), vec!["A".to_string()]);
        assert!(graph.has_dependents("B"));
        assert!(!graph.has_dependents("A"));
    }

    #[test]
    fn load_order() {
        let mut graph = ResourceDependencyGraph::new();
        // A depends on B, B depends on C
        graph.register_dependency("A", "B");
        graph.register_dependency("B", "C");

        let order = graph.load_order(&["A".to_string()]);
        assert_eq!(order, vec!["C", "B", "A"]);
    }

    #[test]
    fn remove_cleans_both_directions() {
        let mut graph = ResourceDependencyGraph::new();
        graph.register_dependency("A", "B");
        graph.remove("A");
        assert_eq!(graph.dependencies_of("A"), Vec::<String>::new());
        assert!(!graph.has_dependents("B"));
    }
}