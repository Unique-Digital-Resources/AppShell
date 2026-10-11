//! Tests for the tagging utility.

use app_shell::domain_foundation::utilities::tagging::*;

#[test]
fn tag_creation_and_display() {
    let t = Tag::new("important");
    assert_eq!(t.as_str(), "important");
    assert_eq!(format!("{t}"), "#important");
}

#[test]
fn tag_from_str() {
    let t: Tag = "urgent".into();
    assert_eq!(t.as_str(), "urgent");
}

#[test]
fn tag_set_add_remove_contains() {
    let mut set = TagSet::new();
    set.add(Tag::new("a"));
    set.add(Tag::new("b"));
    assert_eq!(set.len(), 2);
    assert!(set.contains(&Tag::new("a")));
    assert!(!set.contains(&Tag::new("c")));

    assert!(set.remove(&Tag::new("a")));
    assert!(!set.contains(&Tag::new("a")));
    assert_eq!(set.len(), 1);
}

#[test]
fn tag_set_intersection_and_union() {
    let mut a = TagSet::new();
    a.add(Tag::new("x"));
    a.add(Tag::new("y"));

    let mut b = TagSet::new();
    b.add(Tag::new("y"));
    b.add(Tag::new("z"));

    let inter = a.intersection(&b);
    assert_eq!(inter.len(), 1);
    assert!(inter.contains(&Tag::new("y")));

    let union = a.union(&b);
    assert_eq!(union.len(), 3);
}

#[test]
fn contains_any_and_all() {
    let mut set = TagSet::new();
    set.add(Tag::new("a"));
    set.add(Tag::new("b"));
    set.add(Tag::new("c"));

    let mut subset = TagSet::new();
    subset.add(Tag::new("a"));
    subset.add(Tag::new("b"));

    let mut other = TagSet::new();
    other.add(Tag::new("x"));

    assert!(set.contains_all(&subset));
    assert!(!set.contains_all(&other));
    assert!(set.contains_any(&subset));
    assert!(!set.contains_any(&other));
}

#[test]
fn tag_set_into_iterator() {
    let mut set = TagSet::new();
    set.add(Tag::new("c"));
    set.add(Tag::new("a"));
    set.add(Tag::new("b"));

    // BTreeSet gives sorted order.
    let names: Vec<&str> = (&set).into_iter().map(|t| t.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}