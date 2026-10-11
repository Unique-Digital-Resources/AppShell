//! Model and query tests for the Canvas2D environment.

use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, Object, ObjectId, Transform, Vec2,
};

#[test]
fn new_canvas_is_empty() {
    let canvas = Canvas::new();
    assert!(canvas.is_empty());
    assert_eq!(canvas.len(), 0);
}

#[test]
fn default_canvas_is_empty() {
    let canvas = Canvas::default();
    assert!(canvas.is_empty());
}

#[test]
fn create_object_returns_unique_ids() {
    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    let c = canvas.create_object(Transform::identity());
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

#[test]
fn created_object_is_queryable_with_its_transform() {
    let mut canvas = Canvas::new();
    let transform = Transform::from_translation(Vec2::new(10.0, 20.0));
    let id = canvas.create_object(transform);

    let obj = canvas.get(id).expect("object must exist immediately after creation");
    assert_eq!(obj.id, id);
    assert_eq!(obj.transform, transform);
}

#[test]
fn contains_reports_membership() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(Transform::identity());
    assert!(canvas.contains(id));
    canvas.remove_object(id).unwrap();
    assert!(!canvas.contains(id));
}

#[test]
fn iter_visits_all_objects() {
    let mut canvas = Canvas::new();
    let _ = canvas.create_object(Transform::identity());
    let _ = canvas.create_object(Transform::identity());
    let _ = canvas.create_object(Transform::identity());
    assert_eq!(canvas.iter().count(), 3);
}

#[test]
fn clear_empties_canvas_but_keeps_id_counter_unique() {
    let mut canvas = Canvas::new();
    let first = canvas.create_object(Transform::identity());
    let _ = canvas.create_object(Transform::identity());
    canvas.clear();
    assert!(canvas.is_empty());

    // After clear, the counter must keep advancing so previously-issued ids
    // cannot collide with new ones.
    let after = canvas.create_object(Transform::identity());
    assert_ne!(first, after);
}

#[test]
fn remove_unknown_id_returns_object_not_found() {
    let mut canvas = Canvas::new();
    let real = canvas.create_object(Transform::identity());
    canvas.remove_object(real).unwrap();

    let err = canvas.remove_object(real).unwrap_err();
    assert!(matches!(err, CanvasError::ObjectNotFound { .. }));
}

#[test]
fn canvas_partial_eq_works() {
    let mut a = Canvas::new();
    let mut b = Canvas::new();

    // Two empty canvases are equal.
    assert_eq!(a, b);

    // Both create identical objects (same id since both start at 0).
    a.create_object(Transform::from_translation(Vec2::new(1.0, 2.0)));
    b.create_object(Transform::from_translation(Vec2::new(1.0, 2.0)));

    // Still equal — same content, same next_id.
    assert_eq!(a, b);

    // Add a second object to one — now they differ.
    a.create_object(Transform::from_translation(Vec2::new(10.0, 20.0)));
    assert_ne!(a, b);
}

#[test]
fn vec2_dot_product() {
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::new(2.0, 0.0);
    assert_eq!(a.dot(b), 6.0);
}

#[test]
fn vec2_length() {
    assert_eq!(Vec2::new(3.0, 4.0).length(), 5.0);
    assert_eq!(Vec2::zero().length(), 0.0);
}

#[test]
fn vec2_normalize() {
    let n = Vec2::new(3.0, 4.0).normalize();
    assert!((n.length() - 1.0).abs() < 1e-10);
    assert_eq!(Vec2::zero().normalize(), Vec2::zero());
}

#[test]
fn vec2_lerp() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 20.0);
    assert_eq!(a.lerp(b, 0.0), a);
    assert_eq!(a.lerp(b, 1.0), b);
    assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 10.0));
}

#[test]
fn vec2_rotate() {
    let v = Vec2::new(1.0, 0.0);
    // Rotate 90 degrees CCW → (0, 1)
    let rotated = v.rotate(std::f64::consts::FRAC_PI_2);
    assert!((rotated.x - 0.0).abs() < 1e-10);
    assert!((rotated.y - 1.0).abs() < 1e-10);

    // Rotate 180 degrees → (-1, 0)
    let rotated = v.rotate(std::f64::consts::PI);
    assert!((rotated.x - (-1.0)).abs() < 1e-10);
    assert!((rotated.y - 0.0).abs() < 1e-10);
}

#[test]
fn vec2_angle_to() {
    let a = Vec2::new(1.0, 0.0);
    let b = Vec2::new(0.0, 1.0);
    // 90 degrees
    assert!((a.angle_to(b) - std::f64::consts::FRAC_PI_2).abs() < 1e-10);

    // Same direction → 0
    assert!((a.angle_to(a)).abs() < 1e-10);

    // Opposite direction → π
    let c = Vec2::new(-1.0, 0.0);
    assert!((a.angle_to(c) - std::f64::consts::PI).abs() < 1e-10);

    // Negative angle (clockwise)
    let d = Vec2::new(0.0, -1.0);
    assert!((a.angle_to(d) + std::f64::consts::FRAC_PI_2).abs() < 1e-10);
}
#[test]
fn canvas_into_iterator() {
    let mut canvas = Canvas::new();
    canvas.create_object(Transform::identity());
    canvas.create_object(Transform::identity());

    let count = (&canvas).into_iter().count();
    assert_eq!(count, 2);
}

#[test]
fn canvas_extend() {
    let mut canvas = Canvas::new();
    let obj1 = Object::new(
        app_shell::domain_foundation::environments::canvas_2d::ObjectId::from_raw(100),
        Transform::identity(),
    );
    let obj2 = Object::new(
        app_shell::domain_foundation::environments::canvas_2d::ObjectId::from_raw(101),
        Transform::identity(),
    );

    canvas.extend([obj1, obj2]);
    assert_eq!(canvas.len(), 2);
    assert_eq!(canvas.draw_order().len(), 2);
}
