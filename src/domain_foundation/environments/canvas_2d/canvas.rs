//! Canvas model and object management.
//!
//! The canvas owns a map of [`Object`]s keyed by stable [`ObjectId`]s,
//! plus a `draw_order` vector for z-ordering (back to front).

use std::collections::HashMap;

use super::error::CanvasError;
use super::object::{Object, ObjectId};
use super::transform::{Transform, Vec2};

/// Headless 2D spatial canvas.
///
/// Objects are stored in a `HashMap` for O(1) lookup and a `Vec<ObjectId>`
/// for draw order (z-ordering). The draw order is independent of
/// insertion order — use `move_to_front` / `move_to_back` to control it.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Canvas {
    objects: HashMap<ObjectId, Object>,
    draw_order: Vec<ObjectId>,
    next_id: u64,
}

impl Canvas {
    /// Create an empty canvas.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of objects currently on the canvas.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn contains(&self, id: ObjectId) -> bool {
        self.objects.contains_key(&id)
    }

    /// Look up an object by id.
    pub fn get(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(&id)
    }

    /// Mutable lookup by id.
    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(&id)
    }

    /// Iterate over all objects. Order is unspecified (HashMap order).
    /// Use [`draw_order`](Self::draw_order) for z-ordered iteration.
    pub fn iter(&self) -> impl Iterator<Item = &Object> {
        self.objects.values()
    }

    /// Object ids in draw order (back to front).
    pub fn draw_order(&self) -> &[ObjectId] {
        &self.draw_order
    }

    /// Iterate over objects in draw order (back to front).
    pub fn iter_ordered(&self) -> impl Iterator<Item = &Object> {
        self.draw_order
            .iter()
            .filter_map(move |id| self.objects.get(id))
    }

    /// Insert a new object with the given transform. Returns its assigned id.
    /// The object is placed at the front of the draw order.
    pub fn create_object(&mut self, transform: Transform) -> ObjectId {
        let id = ObjectId::from_raw(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.objects.insert(id, Object::new(id, transform));
        self.draw_order.push(id);
        id
    }

    /// Insert an existing object (for undo of RemoveObject). Restores
    /// the object with its original id. Also restores its draw-order
    /// position if not already present.
    pub fn insert_object(&mut self, object: Object) {
        let id = object.id;
        if id.as_raw() >= self.next_id {
            self.next_id = id.as_raw() + 1;
        }
        if !self.draw_order.contains(&id) {
            self.draw_order.push(id);
        }
        self.objects.insert(id, object);
    }

    /// Replace an object's transform wholesale. Returns the previous
    /// transform for undo.
    pub fn set_transform(
        &mut self,
        id: ObjectId,
        transform: Transform,
    ) -> Result<Transform, CanvasError> {
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        let previous = obj.transform;
        obj.transform = transform;
        Ok(previous)
    }

    /// Set only the translation component, preserving rotation and scale.
    pub fn set_translation(
        &mut self,
        id: ObjectId,
        translation: Vec2,
    ) -> Result<Transform, CanvasError> {
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        let previous = obj.transform;
        obj.transform.translation = translation;
        Ok(previous)
    }

    /// Remove an object. Returns the removed object for undo.
    pub fn remove_object(&mut self, id: ObjectId) -> Result<Object, CanvasError> {
        let obj = self
            .objects
            .remove(&id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        self.draw_order.retain(|oid| *oid != id);
        Ok(obj)
    }

    /// Remove all objects. The id counter is not reset.
    pub fn clear(&mut self) {
        self.objects.clear();
        self.draw_order.clear();
    }

    // --- Z-ordering (gap #6) ---

    /// Move an object to the front (topmost in draw order).
    pub fn move_to_front(&mut self, id: ObjectId) -> Result<(), CanvasError> {
        if !self.objects.contains_key(&id) {
            return Err(CanvasError::ObjectNotFound { id });
        }
        self.draw_order.retain(|oid| *oid != id);
        self.draw_order.push(id);
        Ok(())
    }

    /// Move an object to the back (bottommost in draw order).
    pub fn move_to_back(&mut self, id: ObjectId) -> Result<(), CanvasError> {
        if !self.objects.contains_key(&id) {
            return Err(CanvasError::ObjectNotFound { id });
        }
        self.draw_order.retain(|oid| *oid != id);
        self.draw_order.insert(0, id);
        Ok(())
    }

    /// Move an object forward by one position in the draw order.
    pub fn move_forward(&mut self, id: ObjectId) -> Result<(), CanvasError> {
        if !self.objects.contains_key(&id) {
            return Err(CanvasError::ObjectNotFound { id });
        }
        let pos = self
            .draw_order
            .iter()
            .position(|oid| *oid == id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        if pos < self.draw_order.len() - 1 {
            self.draw_order.swap(pos, pos + 1);
        }
        Ok(())
    }

    /// Move an object backward by one position in the draw order.
    pub fn move_backward(&mut self, id: ObjectId) -> Result<(), CanvasError> {
        if !self.objects.contains_key(&id) {
            return Err(CanvasError::ObjectNotFound { id });
        }
        let pos = self
            .draw_order
            .iter()
            .position(|oid| *oid == id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        if pos > 0 {
            self.draw_order.swap(pos, pos - 1);
        }
        Ok(())
    }

    /// Reorder an object to a specific index in the draw order.
    pub fn reorder(&mut self, id: ObjectId, new_index: usize) -> Result<(), CanvasError> {
        if !self.objects.contains_key(&id) {
            return Err(CanvasError::ObjectNotFound { id });
        }
        let current = self
            .draw_order
            .iter()
            .position(|oid| *oid == id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        if new_index >= self.draw_order.len() {
            return Err(CanvasError::ObjectNotFound { id });
        }
        if current == new_index {
            return Ok(());
        }
        self.draw_order.remove(current);
        self.draw_order.insert(new_index, id);
        Ok(())
    }

    // --- Spatial queries (gap #15) ---

    /// Find all objects whose translation falls within the given bounds
    /// (inclusive min, exclusive max).
    pub fn objects_in_bounds(&self, min: Vec2, max: Vec2) -> Vec<&Object> {
        self.objects
            .values()
            .filter(|obj| {
                let p = obj.transform.translation;
                p.x >= min.x && p.x < max.x && p.y >= min.y && p.y < max.y
            })
            .collect()
    }

    /// Create a duplicate of an object at the given offset.
    /// Returns the new object's id.
    pub fn duplicate_object(
        &mut self,
        id: ObjectId,
        offset: Vec2,
    ) -> Result<ObjectId, CanvasError> {
        let obj = self
            .objects
            .get(&id)
            .ok_or(CanvasError::ObjectNotFound { id })?;
        let new_transform = Transform {
            translation: Vec2::new(
                obj.transform.translation.x + offset.x,
                obj.transform.translation.y + offset.y,
            ),
            rotation: obj.transform.rotation,
            scale: obj.transform.scale,
        };
        Ok(self.create_object(new_transform))
    }

}

impl<'a> IntoIterator for &'a Canvas {
    type Item = &'a Object;
    type IntoIter = std::collections::hash_map::Values<'a, ObjectId, Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.values()
    }
}

impl Extend<Object> for Canvas {
    fn extend<T: IntoIterator<Item = Object>>(&mut self, iter: T) {
        for obj in iter {
            if obj.id.as_raw() >= self.next_id {
                self.next_id = obj.id.as_raw() + 1;
            }
            if !self.draw_order.contains(&obj.id) {
                self.draw_order.push(obj.id);
            }
            self.objects.insert(obj.id, obj);
        }
    }
}