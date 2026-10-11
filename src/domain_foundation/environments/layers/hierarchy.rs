//! Layer hierarchy: owns all layers, manages parent-child relationships,
//! sibling ordering, and root-level tracking.

use super::error::LayersError;
use super::layer::{Layer, LayerId};
use std::collections::{HashMap, VecDeque};

/// Headless layer hierarchy.
///
/// Owns a set of [`Layer`]s organised as a tree. Each layer has at most
/// one parent. Root layers (no parent) are tracked in an ordered list.
/// Siblings within a parent are also ordered.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layers {
    layers: HashMap<LayerId, Layer>,
    roots: Vec<LayerId>,
    next_id: u64,
}

impl Layers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn contains(&self, id: LayerId) -> bool {
        self.layers.contains_key(&id)
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(&id)
    }

    pub fn roots(&self) -> &[LayerId] {
        &self.roots
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// Create a root-level layer.
    pub fn create_layer(&mut self, name: String) -> LayerId {
        let id = LayerId::from_raw(self.next_id);
        self.next_id += 1;
        let layer = Layer::new(id, name);
        self.roots.push(id);
        self.layers.insert(id, layer);
        id
    }

    /// Create a child of an existing layer.
    pub fn create_child(&mut self, parent: LayerId, name: String) -> Result<LayerId, LayersError> {
        if !self.layers.contains_key(&parent) {
            return Err(LayersError::LayerNotFound { id: parent });
        }
        let id = LayerId::from_raw(self.next_id);
        self.next_id += 1;
        let mut layer = Layer::new(id, name);
        layer.parent = Some(parent);
        self.layers.insert(id, layer);
        self.layers.get_mut(&parent).unwrap().children.push(id);
        Ok(id)
    }

    /// Insert an existing layer (for undo of RemoveLayer). Restores
    /// the layer with its original id, parent, and children. The `contains`
    /// check prevents double-adding to a parent's children list when
    /// re-inserting a subtree layer by layer.
    pub fn insert_layer(&mut self, layer: Layer) -> LayerId {
        let id = layer.id;
        if id.as_raw() >= self.next_id {
            self.next_id = id.as_raw() + 1;
        }
        match layer.parent {
            Some(parent) => {
                if let Some(p) = self.layers.get_mut(&parent) {
                    if !p.children.contains(&id) {
                        p.children.push(id);
                    }
                }
            }
            None => {
                if !self.roots.contains(&id) {
                    self.roots.push(id);
                }
            }
        }
        self.layers.insert(id, layer);
        id
    }

    /// Remove a layer and all its descendants. Returns all removed
    /// layers (the layer itself plus descendants, in BFS order) for undo.
    pub fn remove_layer(&mut self, id: LayerId) -> Result<Vec<Layer>, LayersError> {
        if !self.layers.contains_key(&id) {
            return Err(LayersError::LayerNotFound { id });
        }

        // Collect all descendants via BFS (parent before children).
        let mut to_remove: Vec<LayerId> = vec![id];
        let mut queue = VecDeque::new();
        queue.push_back(id);
        while let Some(current) = queue.pop_front() {
            if let Some(layer) = self.layers.get(&current) {
                for &child in layer.children() {
                    to_remove.push(child);
                    queue.push_back(child);
                }
            }
        }

        // Remove from parent's children list or roots.
        let parent = self.layers.get(&id).map(|l| l.parent).flatten();
        match parent {
            Some(p) => {
                if let Some(p_layer) = self.layers.get_mut(&p) {
                    p_layer.children.retain(|c| *c != id);
                }
            }
            None => {
                self.roots.retain(|r| *r != id);
            }
        }

        // Remove all collected layers from the HashMap.
        let mut removed = Vec::new();
        for rid in &to_remove {
            if let Some(l) = self.layers.remove(rid) {
                removed.push(l);
            }
        }

        Ok(removed)
    }

    /// Set visibility. Returns the previous value.
    pub fn set_visibility(&mut self, id: LayerId, visible: bool) -> Result<bool, LayersError> {
        let layer = self.layers.get_mut(&id).ok_or(LayersError::LayerNotFound { id })?;
        let previous = layer.visible;
        layer.visible = visible;
        Ok(previous)
    }

    /// Set locked state. Returns the previous value.
    pub fn set_locked(&mut self, id: LayerId, locked: bool) -> Result<bool, LayersError> {
        let layer = self.layers.get_mut(&id).ok_or(LayersError::LayerNotFound { id })?;
        let previous = layer.locked;
        layer.locked = locked;
        Ok(previous)
    }

    /// Rename a layer. Returns the previous name.
    pub fn rename(&mut self, id: LayerId, name: String) -> Result<String, LayersError> {
        let layer = self.layers.get_mut(&id).ok_or(LayersError::LayerNotFound { id })?;
        let previous = layer.name.clone();
        layer.name = name;
        Ok(previous)
    }

    /// Reorder a layer within its siblings. Returns the previous index.
    pub fn reorder_sibling(&mut self, id: LayerId, new_index: usize) -> Result<usize, LayersError> {
        if !self.layers.contains_key(&id) {
            return Err(LayersError::LayerNotFound { id });
        }

        let parent = self.layers.get(&id).map(|l| l.parent).flatten();

        // Find current index and sibling count (immutable borrow).
        let (current_index, sibling_count) = match parent {
            Some(p) => {
                let children = &self.layers.get(&p).unwrap().children;
                let idx = children.iter().position(|c| *c == id)
                    .ok_or(LayersError::LayerNotFound { id })?;
                (idx, children.len())
            }
            None => {
                let idx = self.roots.iter().position(|r| *r == id)
                    .ok_or(LayersError::LayerNotFound { id })?;
                (idx, self.roots.len())
            }
        };

        if new_index >= sibling_count {
            return Err(LayersError::InvalidSiblingIndex {
                index: new_index,
                sibling_count,
            });
        }

        if current_index == new_index {
            return Ok(current_index);
        }

        // Perform the reorder (mutable borrow — immutable borrow has ended).
        match parent {
            Some(p) => {
                let children = &mut self.layers.get_mut(&p).unwrap().children;
                children.remove(current_index);
                children.insert(new_index, id);
            }
            None => {
                self.roots.remove(current_index);
                self.roots.insert(new_index, id);
            }
        }

        Ok(current_index)
    }

    /// Move a layer to a different parent. Returns the previous parent.
    /// `new_parent = None` makes the layer a root.
    /// Rejects cycles (new parent must not be the layer itself or a
    /// descendant of the layer).
    pub fn reparent(
        &mut self,
        id: LayerId,
        new_parent: Option<LayerId>,
    ) -> Result<Option<LayerId>, LayersError> {
        if !self.layers.contains_key(&id) {
            return Err(LayersError::LayerNotFound { id });
        }

        if let Some(np) = new_parent {
            if !self.layers.contains_key(&np) {
                return Err(LayersError::LayerNotFound { id: np });
            }
            if np == id || self.is_descendant(np, id) {
                return Err(LayersError::CycleDetected { id, new_parent: np });
            }
        }

        let old_parent = self.layers.get(&id).map(|l| l.parent).flatten();

        // Remove from old parent's children or roots.
        match old_parent {
            Some(op) => {
                if let Some(p) = self.layers.get_mut(&op) {
                    p.children.retain(|c| *c != id);
                }
            }
            None => {
                self.roots.retain(|r| *r != id);
            }
        }

        // Add to new parent's children or roots.
        match new_parent {
            Some(np) => {
                self.layers.get_mut(&np).unwrap().children.push(id);
            }
            None => {
                self.roots.push(id);
            }
        }

        self.layers.get_mut(&id).unwrap().parent = new_parent;
        Ok(old_parent)
    }

    /// Check if `maybe_descendant` is a descendant of `ancestor`
    /// (or the same layer).
    pub fn is_descendant(&self, maybe_descendant: LayerId, ancestor: LayerId) -> bool {
        if maybe_descendant == ancestor {
            return true;
        }
        let mut to_check = VecDeque::new();
        to_check.push_back(ancestor);
        while let Some(current) = to_check.pop_front() {
            if let Some(layer) = self.layers.get(&current) {
                for &child in layer.children() {
                    if child == maybe_descendant {
                        return true;
                    }
                    to_check.push_back(child);
                }
            }
        }
        false
    }

    /// All descendants of a layer (not including the layer itself).
    pub fn descendants(&self, id: LayerId) -> Vec<LayerId> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(id);
        while let Some(current) = queue.pop_front() {
            if let Some(layer) = self.layers.get(&current) {
                for &child in layer.children() {
                    result.push(child);
                    queue.push_back(child);
                }
            }
        }
        result
    }

    /// Maximum depth of the hierarchy (longest root-to-leaf chain).
    pub fn depth(&self) -> usize {
        self.roots.iter().map(|r| self.subtree_depth(*r)).max().unwrap_or(0)
    }

    fn subtree_depth(&self, id: LayerId) -> usize {
        match self.layers.get(&id) {
            Some(layer) if !layer.children.is_empty() => {
                1 + layer.children.iter()
                    .map(|c| self.subtree_depth(*c))
                    .max()
                    .unwrap_or(0)
            }
            Some(_) => 1,
            None => 0,
        }
    }

    /// Compute effective visibility: a layer is effectively visible
    /// only if it AND all its ancestors are visible.
    pub fn effective_visibility(&self, id: LayerId) -> bool {
        let mut current = id;
        loop {
            let layer = match self.layers.get(&current) {
                Some(l) => l,
                None => return true,
            };
            if !layer.visible {
                return false;
            }
            match layer.parent {
                Some(parent) => current = parent,
                None => return true,
            }
        }
    }

    /// Remove all layers. Id counter is not reset.
    pub fn clear(&mut self) {
        self.layers.clear();
        self.roots.clear();
    }

    /// Compute effective locked state: a layer is effectively locked
    /// if it OR any ancestor is locked.
    pub fn effective_locked(&self, id: LayerId) -> bool {
        let mut current = id;
        loop {
            let layer = match self.layers.get(&current) {
                Some(l) => l,
                None => return false,
            };
            if layer.locked {
                return true;
            }
            match layer.parent {
                Some(parent) => current = parent,
                None => return false,
            }
        }
    }

    /// Path from root to this layer, inclusive. Returns `[]` if not found.
    pub fn layer_path(&self, id: LayerId) -> Vec<LayerId> {
        let mut path = Vec::new();
        let mut current = id;
        loop {
            let layer = match self.layers.get(&current) {
                Some(l) => l,
                None => break,
            };
            path.push(current);
            match layer.parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
        path.reverse(); // root first, leaf last
        path
    }


    /// Move a layer one position forward in its sibling list.
    /// Returns `Ok(true)` if moved, `Ok(false)` if already at front.
    pub fn move_up(&mut self, id: LayerId) -> Result<bool, LayersError> {
        if !self.layers.contains_key(&id) {
            return Err(LayersError::LayerNotFound { id });
        }
        let parent = self.layers.get(&id).map(|l| l.parent).flatten();
        let siblings: &mut Vec<LayerId> = match parent {
            Some(p) => &mut self.layers.get_mut(&p).unwrap().children,
            None => &mut self.roots,
        };
        let pos = siblings
            .iter()
            .position(|s| *s == id)
            .ok_or(LayersError::LayerNotFound { id })?;
        if pos == 0 {
            return Ok(false);
        }
        siblings.swap(pos, pos - 1);
        Ok(true)
    }

    /// Move a layer one position backward in its sibling list.
    /// Returns `Ok(true)` if moved, `Ok(false)` if already at back.
    pub fn move_down(&mut self, id: LayerId) -> Result<bool, LayersError> {
        if !self.layers.contains_key(&id) {
            return Err(LayersError::LayerNotFound { id });
        }
        let parent = self.layers.get(&id).map(|l| l.parent).flatten();
        let siblings: &mut Vec<LayerId> = match parent {
            Some(p) => &mut self.layers.get_mut(&p).unwrap().children,
            None => &mut self.roots,
        };
        let pos = siblings
            .iter()
            .position(|s| *s == id)
            .ok_or(LayersError::LayerNotFound { id })?;
        if pos >= siblings.len() - 1 {
            return Ok(false);
        }
        siblings.swap(pos, pos + 1);
        Ok(true)
    }

    /// Duplicate a layer as a new root. The duplicate gets a new id and
    /// no parent. Children are NOT duplicated.
    pub fn duplicate_layer(&mut self, id: LayerId) -> Result<LayerId, LayersError> {
        let layer = self.layers.get(&id)
            .ok_or(LayersError::LayerNotFound { id })?;
        let new_id = LayerId::from_raw(self.next_id);
        self.next_id += 1;
        let mut dup = Layer::new(new_id, layer.name.clone());
        dup.visible = layer.visible;
        dup.locked = layer.locked;
        self.roots.push(new_id);
        self.layers.insert(new_id, dup);
        Ok(new_id)
    }

}