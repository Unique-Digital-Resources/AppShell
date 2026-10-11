//! Model and query tests for the Viewport3D environment.

use app_shell::domain_foundation::environments::viewport_3d::{
    ObjectId3D, Scene, Transform3D, Vec3, Viewport3DError,
};

#[test]
fn new_scene_is_empty() {
    let scene = Scene::new();
    assert!(scene.is_empty());
    assert_eq!(scene.len(), 0);
}

#[test]
fn create_object_returns_unique_ids() {
    let mut scene = Scene::new();
    let a = scene.create_object(Transform3D::identity());
    let b = scene.create_object(Transform3D::identity());
    let c = scene.create_object(Transform3D::identity());
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
    assert_eq!(scene.len(), 3);
}

#[test]
fn created_object_is_queryable_with_its_transform() {
    let mut scene = Scene::new();
    let transform = Transform3D::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let id = scene.create_object(transform);

    let obj = scene.get(id).expect("object must exist after creation");
    assert_eq!(obj.id, id);
    assert_eq!(obj.transform, transform);
}

#[test]
fn contains_reports_membership() {
    let mut scene = Scene::new();
    let id = scene.create_object(Transform3D::identity());
    assert!(scene.contains(id));
    scene.remove_object(id).unwrap();
    assert!(!scene.contains(id));
}

#[test]
fn iter_visits_all_objects() {
    let mut scene = Scene::new();
    let _ = scene.create_object(Transform3D::identity());
    let _ = scene.create_object(Transform3D::identity());
    let _ = scene.create_object(Transform3D::identity());
    assert_eq!(scene.iter().count(), 3);
}

#[test]
fn clear_empties_scene_but_keeps_id_counter_unique() {
    let mut scene = Scene::new();
    let first = scene.create_object(Transform3D::identity());
    let _ = scene.create_object(Transform3D::identity());
    scene.clear();
    assert!(scene.is_empty());

    let after = scene.create_object(Transform3D::identity());
    assert_ne!(first, after);
}

#[test]
fn remove_unknown_id_returns_object_not_found() {
    let mut scene = Scene::new();
    let real = scene.create_object(Transform3D::identity());
    scene.remove_object(real).unwrap();

    let err = scene.remove_object(real).unwrap_err();
    assert!(matches!(err, Viewport3DError::ObjectNotFound { .. }));
}

#[test]
fn set_translation_preserves_rotation_and_scale() {
    let mut scene = Scene::new();
    let id = scene.create_object(
        Transform3D::identity()
            .with_rotation(Vec3::new(0.5, 0.0, 0.0))
            .with_scale(Vec3::new(2.0, 2.0, 2.0)),
    );

    scene.set_translation(id, Vec3::new(10.0, 20.0, 30.0)).unwrap();

    let obj = scene.get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec3::new(10.0, 20.0, 30.0));
    assert_eq!(obj.transform.rotation, Vec3::new(0.5, 0.0, 0.0));
    assert_eq!(obj.transform.scale, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn set_rotation_preserves_translation_and_scale() {
    let mut scene = Scene::new();
    let id = scene.create_object(
        Transform3D::from_translation(Vec3::new(1.0, 2.0, 3.0))
            .with_scale(Vec3::new(2.0, 2.0, 2.0)),
    );

    scene.set_rotation(id, Vec3::new(0.1, 0.2, 0.3)).unwrap();

    let obj = scene.get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(obj.transform.rotation, Vec3::new(0.1, 0.2, 0.3));
    assert_eq!(obj.transform.scale, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn set_scale_preserves_translation_and_rotation() {
    let mut scene = Scene::new();
    let id = scene.create_object(
        Transform3D::from_translation(Vec3::new(1.0, 2.0, 3.0))
            .with_rotation(Vec3::new(0.5, 0.0, 0.0)),
    );

    scene.set_scale(id, Vec3::new(3.0, 4.0, 5.0)).unwrap();

    let obj = scene.get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(obj.transform.rotation, Vec3::new(0.5, 0.0, 0.0));
    assert_eq!(obj.transform.scale, Vec3::new(3.0, 4.0, 5.0));
}

#[test]
fn insert_object_restores_with_original_id() {
    let mut scene = Scene::new();
    let id = scene.create_object(Transform3D::from_translation(Vec3::new(7.0, 8.0, 9.0)));
    let obj = scene.remove_object(id).unwrap();
    assert!(!scene.contains(id));

    scene.insert_object(obj);
    assert!(scene.contains(id));
    assert_eq!(scene.get(id).unwrap().transform.translation, Vec3::new(7.0, 8.0, 9.0));
}

#[test]
fn vec3_dot_cross_and_length() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);

    assert_eq!(a.dot(b), 0.0);
    assert_eq!(a.cross(b), Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(a.length(), 1.0);

    let c = Vec3::new(3.0, 4.0, 0.0);
    assert_eq!(c.length(), 5.0);
}

#[test]
fn vec3_normalised() {
    let v = Vec3::new(3.0, 4.0, 0.0);
    let n = v.normalised();
    assert!((n.length() - 1.0).abs() < 1e-10);
}


#[test]
fn parent_child_transform_hierarchy() {
    let mut scene = Scene::new();
    let parent = scene.create_object(Transform3D::from_translation(Vec3::new(10.0, 0.0, 0.0)));
    let child = scene.create_object(Transform3D::from_translation(Vec3::new(5.0, 0.0, 0.0)));
    let grandchild = scene.create_object(Transform3D::from_translation(Vec3::new(1.0, 0.0, 0.0)));

    scene.set_parent(child, Some(parent)).unwrap();
    scene.set_parent(grandchild, Some(child)).unwrap();

    // World transform: 10 + 5 + 1 = 16 on X.
    let world = scene.world_transform(grandchild).unwrap();
    assert_eq!(world.translation.x, 16.0);

    // Child's world: 10 + 5 = 15.
    let world_child = scene.world_transform(child).unwrap();
    assert_eq!(world_child.translation.x, 15.0);
}

#[test]
fn set_parent_rejects_cycle() {
    let mut scene = Scene::new();
    let a = scene.create_object(Transform3D::identity());
    let b = scene.create_object(Transform3D::identity());

    scene.set_parent(b, Some(a)).unwrap();

    // Setting a as child of b would create a cycle.
    let err = scene.set_parent(a, Some(b)).unwrap_err();
    assert!(matches!(err, app_shell::domain_foundation::environments::viewport_3d::Viewport3DError::CannotBeOwnParent { .. }));
}

#[test]
fn scene_partial_eq_works() {
    let a = Scene::new();
    let b = Scene::new();
    assert_eq!(a, b);
}

#[test]
fn scene_objects_in_bounds() {
    let mut scene = Scene::new();
    scene.create_object(Transform3D::from_translation(Vec3::new(0.0, 0.0, 0.0)));
    scene.create_object(Transform3D::from_translation(Vec3::new(10.0, 0.0, 0.0)));
    scene.create_object(Transform3D::from_translation(Vec3::new(20.0, 20.0, 0.0)));
    scene.create_object(Transform3D::from_translation(Vec3::new(-5.0, -5.0, 0.0)));

    let found = scene.objects_in_bounds(
        Vec3::new(-10.0, -10.0, -10.0),
        Vec3::new(15.0, 15.0, 10.0),
    );
    // Objects at (0,0,0), (10,0,0), (-5,-5,0) are in bounds.
    // (20,20,0) is out.
    assert_eq!(found.len(), 3);
}

#[test]
fn scene_children() {
    let mut scene = Scene::new();
    let parent = scene.create_object(Transform3D::identity());
    let child_a = scene.create_object(Transform3D::identity());
    let child_b = scene.create_object(Transform3D::identity());
    let grandchild = scene.create_object(Transform3D::identity());

    scene.set_parent(child_a, Some(parent)).unwrap();
    scene.set_parent(child_b, Some(parent)).unwrap();
    scene.set_parent(grandchild, Some(child_a)).unwrap();

    let children = scene.children(parent);
    assert_eq!(children.len(), 2);

    let grandchildren = scene.children(child_a);
    assert_eq!(grandchildren.len(), 1);
}

#[test]
fn scene_root_objects() {
    let mut scene = Scene::new();
    let a = scene.create_object(Transform3D::identity());
    let b = scene.create_object(Transform3D::identity());
    let c = scene.create_object(Transform3D::identity());

    scene.set_parent(c, Some(a)).unwrap();

    let roots = scene.root_objects();
    // a and b are roots; c is a child of a.
    assert_eq!(roots.len(), 2);
}

#[test]
fn vec3_lerp() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(10.0, 20.0, 30.0);

    assert_eq!(a.lerp(b, 0.0), a);
    assert_eq!(a.lerp(b, 1.0), b);
    assert_eq!(a.lerp(b, 0.5), Vec3::new(5.0, 10.0, 15.0));
}

#[test]
fn vec3_distance_to() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(3.0, 4.0, 0.0);
    assert!((a.distance_to(b) - 5.0).abs() < 1e-10);

    let c = Vec3::new(1.0, 2.0, 3.0);
    let d = Vec3::new(1.0, 2.0, 3.0);
    assert!((c.distance_to(d)).abs() < 1e-10);
}