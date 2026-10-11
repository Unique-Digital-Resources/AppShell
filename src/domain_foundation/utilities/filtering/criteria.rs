//! Filter criteria: composable predicates for evaluating objects.

/// A criterion that can evaluate whether an item matches.
///
/// Implementations are composable via [`and`](Self::and),
/// [`or`](Self::or), and [`not`](Self::not).
pub trait FilterCriteria<T> {
    fn matches(&self, item: &T) -> bool;

    /// Both criteria must match (logical AND).
    fn and<C: FilterCriteria<T>>(self, other: C) -> And<Self, C>
    where
        Self: Sized,
    {
        And { left: self, right: other }
    }

    /// Either criterion must match (logical OR).
    fn or<C: FilterCriteria<T>>(self, other: C) -> Or<Self, C>
    where
        Self: Sized,
    {
        Or { left: self, right: other }
    }

    /// Negate this criterion (logical NOT).
    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not { inner: self }
    }
}

/// Logical AND of two criteria.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct And<C1, C2> {
    pub left: C1,
    pub right: C2,
}

impl<T, C1: FilterCriteria<T>, C2: FilterCriteria<T>> FilterCriteria<T> for And<C1, C2> {
    fn matches(&self, item: &T) -> bool {
        self.left.matches(item) && self.right.matches(item)
    }
}

/// Logical OR of two criteria.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Or<C1, C2> {
    pub left: C1,
    pub right: C2,
}

impl<T, C1: FilterCriteria<T>, C2: FilterCriteria<T>> FilterCriteria<T> for Or<C1, C2> {
    fn matches(&self, item: &T) -> bool {
        self.left.matches(item) || self.right.matches(item)
    }
}

/// Logical NOT of a criterion.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Not<C> {
    pub inner: C,
}

impl<T, C: FilterCriteria<T>> FilterCriteria<T> for Not<C> {
    fn matches(&self, item: &T) -> bool {
        !self.inner.matches(item)
    }
}

/// A closure-based filter criterion.
pub struct FnCriteria<F> {
    f: F,
}

impl<F> std::fmt::Debug for FnCriteria<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FnCriteria(..)")
    }
}

impl<T, F: Fn(&T) -> bool> FilterCriteria<T> for FnCriteria<F> {
    fn matches(&self, item: &T) -> bool {
        (self.f)(item)
    }
}

/// Create a filter criterion from a closure.
pub fn criteria<T, F: Fn(&T) -> bool>(f: F) -> FnCriteria<F> {
    FnCriteria { f }
}