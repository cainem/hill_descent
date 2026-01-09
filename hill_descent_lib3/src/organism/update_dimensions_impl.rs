//! Dimensions update implementation for organisms.

use std::sync::Arc;

use crate::world::dimensions::Dimensions;

/// Updates the organism's dimensions reference.
///
/// Called when dimension bounds expand and organisms need the new reference.
///
/// # Arguments
///
/// * `new_dimensions` - The new dimensions Arc to use
///
/// # Returns
///
/// The new dimensions Arc (for assignment by caller).
///
/// # Note
///
/// This is a simple reference update - the caller handles assigning the result
/// to the organism's dimensions field.
pub fn update_dimensions(new_dimensions: Arc<Dimensions>) -> Arc<Dimensions> {
    // Simply return the new dimensions - caller handles assignment
    new_dimensions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_new_dimensions_when_update_then_returns_same_arc() {
        // This test can run now - it's a simple passthrough
        let dims = Arc::new(Dimensions::default());
        let result = update_dimensions(dims.clone());
        assert!(Arc::ptr_eq(&dims, &result));
    }
}
