use crate::WorldFunction;

use std::fmt::Debug;
/// this trait represents the special case for a function that represents a
/// line or a surface (or even higher dimensions) for which we want to minimize a single
/// value
/// A simple line graph or the height of a surface say is a good example
///
/// Here the expressed values are fed into the function and a value is returned and the
/// algorithm will vary the expressed values to minimize the return value
pub trait SingleValuedFunction: Debug + Sync {
    fn single_run(&self, phenotype_expressed_values: &[f64]) -> f64;

    /// Returns the theoretical minimum value (floor) of this function.
    ///
    /// This is used to validate that computed values are not below the theoretical minimum,
    /// which would indicate a bug in the function implementation.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `0.0`, maintaining backward compatibility with
    /// existing functions that assume a minimum of zero.
    ///
    /// # Examples
    ///
    /// For a function with a known minimum of -5.0:
    /// ```ignore
    /// fn function_floor(&self) -> f64 {
    ///     -5.0
    /// }
    /// ```
    fn function_floor(&self) -> f64 {
        0.0
    }
}

impl<T> WorldFunction for T
where
    T: SingleValuedFunction + Debug,
{
    /// Adapts the `single_run` interface to the `WorldFunction` interface by wrapping the
    /// single scalar result in a `Vec`.
    fn run(&self, phenotype_expressed_values: &[f64], _inputs: &[f64]) -> Vec<f64> {
        vec![self.single_run(phenotype_expressed_values)]
    }

    /// Forwards the floor from SingleValuedFunction to WorldFunction.
    ///
    /// This ensures that when a SingleValuedFunction is used as a WorldFunction,
    /// its custom floor value is preserved and used for validation.
    fn function_floor(&self) -> f64 {
        SingleValuedFunction::function_floor(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock function with default floor (0.0)
    #[derive(Debug)]
    struct DefaultFloorFunction;

    impl SingleValuedFunction for DefaultFloorFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            0.5 // Returns value above default floor of 0.0
        }
    }

    /// Mock function with custom floor (1.0)
    #[derive(Debug)]
    struct CustomFloorFunction;

    impl SingleValuedFunction for CustomFloorFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            1.5 // Returns value above custom floor of 1.0
        }

        fn function_floor(&self) -> f64 {
            1.0 // Override default floor
        }
    }

    /// Mock function with negative floor (-5.0)
    #[derive(Debug)]
    struct NegativeFloorFunction;

    impl SingleValuedFunction for NegativeFloorFunction {
        fn single_run(&self, _params: &[f64]) -> f64 {
            -2.0 // Returns value above floor of -5.0
        }

        fn function_floor(&self) -> f64 {
            -5.0 // Negative floor value
        }
    }

    #[test]
    fn given_default_floor_when_function_floor_called_then_returns_zero() {
        let func = DefaultFloorFunction;
        assert_eq!(SingleValuedFunction::function_floor(&func), 0.0);
    }

    #[test]
    fn given_custom_floor_when_function_floor_called_then_returns_custom_value() {
        let func = CustomFloorFunction;
        assert_eq!(SingleValuedFunction::function_floor(&func), 1.0);
    }

    #[test]
    fn given_negative_floor_when_function_floor_called_then_returns_negative_value() {
        let func = NegativeFloorFunction;
        assert_eq!(SingleValuedFunction::function_floor(&func), -5.0);
    }

    #[test]
    fn given_single_valued_function_when_used_as_world_function_then_floor_is_preserved() {
        // Test that the WorldFunction adapter preserves the floor
        let func = CustomFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        // The floor should be accessible through the WorldFunction trait
        assert_eq!(world_func.function_floor(), 1.0);
    }

    #[test]
    fn given_single_valued_function_when_run_then_returns_vec_with_single_value() {
        let func = DefaultFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        let result = world_func.run(&[1.0, 2.0], &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0.5);
    }

    #[test]
    fn given_custom_floor_function_when_run_then_output_above_floor() {
        let func = CustomFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        let result = world_func.run(&[1.0], &[]);
        assert_eq!(result[0], 1.5);
        assert!(
            result[0] >= world_func.function_floor(),
            "Output {} should be >= floor {}",
            result[0],
            world_func.function_floor()
        );
    }

    #[test]
    fn given_negative_floor_function_when_run_then_output_above_floor() {
        let func = NegativeFloorFunction;
        let world_func: &dyn WorldFunction = &func;

        let result = world_func.run(&[], &[]);
        assert_eq!(result[0], -2.0);
        assert!(
            result[0] >= world_func.function_floor(),
            "Output {} should be >= floor {}",
            result[0],
            world_func.function_floor()
        );
    }
}
