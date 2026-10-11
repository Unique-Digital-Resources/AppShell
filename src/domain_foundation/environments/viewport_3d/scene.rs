//! Scene model: owns 3D objects, issues ids, and provides queries.

use super::error::Viewport3DError;
use super::object::{Object3D, ObjectId3D};
use super::transform::{Transform3D, Vec3};
use std::collections::HashMap;

/// Headless 3D spatial scene.
///
/// A scene owns a set of [`Object3D`]s keyed by stable [`ObjectId3D`]s.
/// It imposes no camera, projection, viewport, or rendering.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    objects: HashMap<ObjectId3D, Object3D>,
    next_id: u64,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn contains(&self, id: ObjectId3D) -> bool {
        self.objects.contains_key(&id)
    }

    pub fn get(&self, id: ObjectId3D) -> Option<&Object3D> {
        self.objects.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjectId3D) -> Option<&mut Object3D> {
        self.objects.get_mut(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Object3D> {
        self.objects.values()
    }

    /// Insert a new object with the given transform. Returns its assigned id.
    pub fn create_object(&mut self, transform: Transform3D) -> ObjectId3D {
        let id = ObjectId3D::from_raw(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.objects.insert(id, Object3D::new(id, transform));
        id
    }

    /// Insert an existing object (for undo of Remove). Restores the
    /// object with its original id.
    pub fn insert_object(&mut self, object: Object3D) {
        let id = object.id;
        if id.as_raw() >= self.next_id {
            self.next_id = id.as_raw() + 1;
        }
        self.objects.insert(id, object);
    }

    /// Replace an object's transform wholesale. Returns the previous
    /// transform for undo.
    pub fn set_transform(
        &mut self,
        id: ObjectId3D,
        transform: Transform3D,
    ) -> Result<Transform3D, Viewport3DError> {
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(Viewport3DError::ObjectNotFound { id })?;
        let previous = obj.transform;
        obj.transform = transform;
        Ok(previous)
    }

    /// Set only the translation component, preserving rotation and scale.
    pub fn set_translation(
        &mut self,
        id: ObjectId3D,
        translation: Vec3,
    ) -> Result<Transform3D, Viewport3DError> {
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(Viewport3DError::ObjectNotFound { id })?;
        let previous = obj.transform;
        obj.transform.translation = translation;
        Ok(previous)
    }

    /// Set only the rotation component, preserving translation and scale.
    pub fn set_rotation(
        &mut self,
        id: ObjectId3D,
        rotation: Vec3,
    ) -> Result<Transform3D, Viewport3DError> {
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(Viewport3DError::ObjectNotFound { id })?;
        let previous = obj.transform;
        obj.transform.rotation = rotation;
        Ok(previous)
    }

    /// Set only the scale component, preserving translation and rotation.
    pub fn set_scale(
        &mut self,
        id: ObjectId3D,
        scale: Vec3,
    ) -> Result<Transform3D, Viewport3DError> {
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(Viewport3DError::ObjectNotFound { id })?;
        let previous = obj.transform;
        obj.transform.scale = scale;
        Ok(previous)
    }

    /// Remove an object. Returns the removed object for undo.
    pub fn remove_object(&mut self, id: ObjectId3D) -> Result<Object3D, Viewport3DError> {
        self.objects
            .remove(&id)
            .ok_or(Viewport3DError::ObjectNotFound { id })
    }

    /// Remove all objects. The id counter is not reset.
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    /// Set an object's parent for transform hierarchy.
    /// Pass `None` to make the object a root.
    pub fn set_parent(
        &mut self,
        id: ObjectId3D,
        parent: Option<ObjectId3D>,
    ) -> Result<(), Viewport3DError> {
        if let Some(p) = parent {
            if !self.objects.contains_key(&p) {
                return Err(Viewport3DError::ParentNotFound { id: p });
            }
            if p == id {
                return Err(Viewport3DError::CannotBeOwnParent { id });
            }
            // Cycle check: parent must not be a descendant of id.
            if self.is_descendant(p, id) {
                return Err(Viewport3DError::CannotBeOwnParent { id });
            }
        }
        let obj = self
            .objects
            .get_mut(&id)
            .ok_or(Viewport3DError::ObjectNotFound { id })?;
        obj.parent = parent;
        Ok(())
    }

    /// Check if `maybe_descendant` is a descendant of `ancestor`.
    fn is_descendant(&self, maybe_descendant: ObjectId3D, ancestor: ObjectId3D) -> bool {
        if maybe_descendant == ancestor {
            return true;
        }
        let mut to_check = vec![ancestor];
        while let Some(current) = to_check.pop() {
            for obj in self.objects.values() {
                if obj.parent == Some(current) {
                    if obj.id == maybe_descendant {
                        return true;
                    }
                    to_check.push(obj.id);
                }
            }
        }
        false
    }

    /// Compute the world transform by walking up the parent chain.
    /// For Phase 12, this composes translations only. A Domain Engine
    /// can implement full matrix composition if needed.
    pub fn world_transform(&self, id: ObjectId3D) -> Result<Transform3D, Viewport3DError> {
        let mut chain = Vec::new();
        let mut current = id;
        loop {
            let obj = self
                .objects
                .get(&current)
                .ok_or(Viewport3DError::ObjectNotFound { id: current })?;
            chain.push(obj.transform);
            match obj.parent {
                Some(parent_id) => current = parent_id,
                None => break,
            }
        }

        let mut world = Transform3D::identity();
        for t in chain.iter().rev() {
            world.translation = world.translation + t.translation;
        }
        Ok(world)
    }


    /// All objects whose translation falls within the given 3D bounds
    /// (inclusive min, exclusive max on each axis).
    pub fn objects_in_bounds(&self, min: Vec3, max: Vec3) -> Vec<&Object3D> {
        self.objects
            .values()
            .filter(|obj| {
                let p = obj.transform.translation;
                p.x >= min.x && p.x < max.x
                    && p.y >= min.y && p.y < max.y
                    && p.z >= min.z && p.z < max.z
            })
            .collect()
    }

    /// Direct children of `id` (objects whose `parent` is `id`).
    pub fn children(&self, id: ObjectId3D) -> Vec<&Object3D> {
        self.objects
            .values()
            .filter(|obj| obj.parent == Some(id))
            .collect()
    }

    /// All root objects (those with `parent == None`).
    pub fn root_objects(&self) -> Vec<&Object3D> {
        self.objects
            .values()
            .filter(|obj| obj.parent.is_none())
            .collect()
    }

}