/// Formats a score value for human-readable display.
///
/// Handles the special case where unscored organisms have `f64::MAX` scores,
/// displaying them as a more user-friendly message instead of the extremely
/// long numeric representation.
///
/// # Arguments
///
/// * `score` - The score to format. Typically from [`World::get_best_score`](super::World::get_best_score)
///
/// # Returns
///
/// A string representation of the score:
/// - Values ≥ 99.9999% of `f64::MAX` → `"<not yet evaluated>"`
/// - Normal values → Formatted with 6 decimal places
///
/// # Examples
///
/// ## Unscored Organism
///
/// ```
/// use hill_descent_lib::format_score;
///
/// let score = f64::MAX;
/// let display = format_score(score);
/// assert_eq!(display, "<not yet evaluated>");
/// ```
///
/// ## Normal Score
///
/// ```
/// use hill_descent_lib::format_score;
///
/// let score = 0.123456789;
/// let display = format_score(score);
/// assert_eq!(display, "0.123457");
/// ```
///
/// ## Monitoring Optimization Progress
///
/// ```
/// use hill_descent_lib::{setup_world, GlobalConstants, SingleValuedFunction, TrainingData, format_score};
/// use std::ops::RangeInclusive;
///
/// #[derive(Debug)]
/// struct SimpleFn;
/// impl SingleValuedFunction for SimpleFn {
///     fn single_run(&self, params: &[f64]) -> f64 {
///         params[0] * params[0] // Minimize x²
///     }
/// }
///
/// let param_range: Vec<RangeInclusive<f64>> = vec![-10.0..=10.0];
/// let constants = GlobalConstants::new(50, 10);
/// let mut world = setup_world(&param_range, constants, Box::new(SimpleFn));
///
/// // Before any training
/// println!("Initial score: {}", format_score(world.get_best_score())); // "<not yet evaluated>"
///
/// // After training
/// world.training_run(TrainingData::None { floor_value: 0.0 });
/// println!("After training: {}", format_score(world.get_best_score())); // e.g. "0.123456"
/// ```
///
/// # Threshold Selection
///
/// The threshold of 99.9999% of `f64::MAX` was chosen to:
/// - Catch `f64::MAX` exactly (unscored organisms)
/// - Avoid false positives with legitimately large (but scored) values
/// - Account for potential floating-point arithmetic variations
///
/// In practice, optimization scores should never approach `f64::MAX` in real
/// problems, so this threshold is extremely safe.
///
/// # See Also
///
/// - [`World::get_best_score`](super::World::get_best_score) - Get the raw score value
/// - [`World::training_run`](super::World::training_run) - Run optimization to score organisms
pub fn format_score(score: f64) -> String {
    // Use 99.9999% of f64::MAX as threshold to catch unscored organisms
    // while being robust to potential floating-point variations
    if score >= f64::MAX * 0.99999 {
        "<not yet evaluated>".to_string()
    } else {
        format!("{:.6}", score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_exact_max_when_format_score_then_returns_not_evaluated() {
        let score = f64::MAX;
        let result = format_score(score);
        assert_eq!(result, "<not yet evaluated>");
    }

    #[test]
    fn given_near_max_when_format_score_then_returns_not_evaluated() {
        let score = f64::MAX * 0.999999; // Well above threshold
        let result = format_score(score);
        assert_eq!(result, "<not yet evaluated>");
    }

    #[test]
    fn given_zero_score_when_format_score_then_returns_formatted_number() {
        let score = 0.0;
        let result = format_score(score);
        assert_eq!(result, "0.000000");
    }

    #[test]
    fn given_positive_score_when_format_score_then_returns_formatted_with_six_decimals() {
        let score = 0.123456789;
        let result = format_score(score);
        assert_eq!(result, "0.123457"); // Rounded to 6 decimals
    }

    #[test]
    fn given_negative_score_when_format_score_then_returns_formatted_number() {
        let score = -123.456789;
        let result = format_score(score);
        assert_eq!(result, "-123.456789");
    }

    #[test]
    fn given_large_but_valid_score_when_format_score_then_returns_formatted_number() {
        let score = 1e10; // 10 billion - large but nowhere near f64::MAX
        let result = format_score(score);
        assert_eq!(result, "10000000000.000000");
    }

    #[test]
    fn given_very_small_score_when_format_score_then_returns_formatted_number() {
        let score = 1e-10; // Very small positive
        let result = format_score(score);
        assert_eq!(result, "0.000000"); // Rounds to 6 decimals
    }

    #[test]
    fn given_score_just_below_threshold_when_format_score_then_returns_formatted_number() {
        // 99.9% of MAX - well below the 99.9999% threshold
        let score = f64::MAX * 0.999;
        let result = format_score(score);
        // Should be formatted as a number (very long), not "<not yet evaluated>"
        assert_ne!(
            result, "<not yet evaluated>",
            "Should format as number, not as unscored"
        );
        assert!(
            result.contains('.'),
            "Should be a formatted number with decimal point"
        );
        assert!(
            result.len() > 300,
            "Should be an extremely long number (f64::MAX * 0.999)"
        );
    }

    #[test]
    fn given_score_at_threshold_boundary_when_format_score_then_consistent_behavior() {
        // Exactly at the threshold
        let score = f64::MAX * 0.99999;
        let result = format_score(score);
        // At the boundary, should be "<not yet evaluated>" (>= comparison)
        assert_eq!(result, "<not yet evaluated>");
    }
}
