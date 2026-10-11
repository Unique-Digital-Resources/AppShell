//! Batch operations: apply an operation to multiple items.

/// An operation that can be applied to a slice of items.
pub trait BatchOperation<T> {
    fn apply(&self, items: &mut [T]);
}

/// Closure-based batch operation.
pub struct FnBatchOperation<F> {
    f: F,
}

impl<T, F: Fn(&mut [T])> BatchOperation<T> for FnBatchOperation<F> {
    fn apply(&self, items: &mut [T]) {
        (self.f)(items);
    }
}

/// Create a batch operation from a closure.
pub fn operation<T, F: Fn(&mut [T])>(f: F) -> FnBatchOperation<F> {
    FnBatchOperation { f }
}

/// Apply `op` to all `items`.
pub fn apply_batch<T, O: BatchOperation<T>>(op: &O, items: &mut [T]) {
    op.apply(items);
}