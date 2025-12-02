use std::fmt::Debug;

/// Trait for advanced multi-output optimization functions with external inputs.
///
/// **Most users should implement [`SingleValuedFunction`](crate::SingleValuedFunction) instead.**
/// This trait is automatically implemented for all types that implement `SingleValuedFunction`.
///
/// `WorldFunction` is designed for complex scenarios where:
/// - The function produces multiple output values (not just a single scalar)
/// - The function needs external input data for each evaluation
/// - You're implementing neural networks, time series prediction, or similar problems
///
/// # Relationship to SingleValuedFunction
///
/// The library provides a blanket implementation that automatically converts any
/// `SingleValuedFunction` to a `WorldFunction`:
///
/// ```ignore
/// impl<T: SingleValuedFunction> WorldFunction for T {
///     fn run(&self, phenotype_expressed_values: &[f64], _inputs: &[f64]) -> Vec<f64> {
///         vec![self.single_run(phenotype_expressed_values)]
///     }
/// }
/// ```
///
/// This means implementing `SingleValuedFunction` is sufficient for most use cases.
///
/// # When to Use WorldFunction Directly
///
/// Implement this trait directly only if:
/// - Your function genuinely needs the `inputs` parameter
/// - You need to return multiple values that are aggregated elsewhere
/// - You're building a specialized evaluation framework
///
/// # Examples
///
/// ## Simple Case (Use SingleValuedFunction Instead)
///
/// ```
/// // ❌ Don't do this:
/// use hill_descent_lib2::WorldFunction;
///
/// #[derive(Debug)]
/// struct SimpleOptimization;
///
/// impl WorldFunction for SimpleOptimization {
///     fn run(&self, params: &[f64], _inputs: &[f64]) -> Vec<f64> {
///         vec![params.iter().map(|x| x * x).sum()]
///     }
/// }
///
/// // ✅ Do this instead:
/// use hill_descent_lib2::SingleValuedFunction;
///
/// #[derive(Debug)]
/// struct BetterOptimization;
///
/// impl SingleValuedFunction for BetterOptimization {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         params.iter().map(|x| x * x).sum()
///     }
/// }
/// ```
///
/// ## Advanced Case (Legitimate WorldFunction Use)
///
/// ```
/// use hill_descent_lib2::WorldFunction;
///
/// #[derive(Debug)]
/// struct TimeSeriesPredictor {
///     // Internal model state
/// }
///
/// impl WorldFunction for TimeSeriesPredictor {
///     fn run(&self, model_params: &[f64], time_series_data: &[f64]) -> Vec<f64> {
///         // model_params: [learning_rate, hidden_size, ...]
///         // time_series_data: actual time series input values
///         
///         // Generate predictions for each time step
///         time_series_data.iter()
///             .map(|input| self.predict(model_params, *input))
///             .collect()
///     }
/// }
///
/// impl TimeSeriesPredictor {
///     fn predict(&self, _params: &[f64], _input: f64) -> f64 {
///         // Model prediction logic
///         0.0
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// Implementations must be `Sync` as they are called concurrently during
/// fitness evaluation across multiple threads.
///
/// # See Also
///
/// - [`SingleValuedFunction`](crate::SingleValuedFunction) - Simpler trait for most optimization problems
/// - [`setup_world`](crate::setup_world) - Accepts both function types
pub trait WorldFunction: Debug + Sync {
    /// Evaluates the function with given parameters and inputs.
    ///
    /// # Parameters
    ///
    /// * `phenotype_expressed_values` - The evolved parameter values being optimized.
    ///   Length matches the number of parameter ranges provided to
    ///   [`setup_world`](crate::setup_world).
    ///
    /// * `inputs` - External input data for the evaluation. For `SingleValuedFunction`
    ///   implementations, this parameter is unused (passed as empty slice).
    ///
    /// # Returns
    ///
    /// A vector of output values. The length and interpretation of outputs is
    /// problem-specific:
    /// - For `SingleValuedFunction`, returns a single-element vector
    /// - For time series, might return one prediction per time step
    /// - For classification, might return class probabilities
    ///
    /// # Examples
    ///
    /// ```
    /// use hill_descent_lib2::WorldFunction;
    ///
    /// #[derive(Debug)]
    /// struct MultiOutput;
    ///
    /// impl WorldFunction for MultiOutput {
    ///     fn run(&self, params: &[f64], inputs: &[f64]) -> Vec<f64> {
    ///         // Generate multiple outputs based on params and inputs
    ///         inputs.iter()
    ///             .map(|&input| params[0] * input + params[1])
    ///             .collect()
    ///     }
    /// }
    /// ```
    fn run(&self, phenotype_expressed_values: &[f64], inputs: &[f64]) -> Vec<f64>;

    /// Returns the minimum possible value (floor) that the function can return.
    ///
    /// This is used for validation during fitness scoring. If the function returns a value
    /// below this floor, it indicates a bug in the function implementation and will cause
    /// a panic.
    ///
    /// # Default Implementation
    ///
    /// Returns `0.0`, which is appropriate for most optimization problems where the optimal
    /// score is zero (perfect match). Override this method if your function has a different
    /// known minimum value.
    ///
    /// # Returns
    ///
    /// The minimum value that this function can validly return.
    fn function_floor(&self) -> f64 {
        0.0
    }
}
