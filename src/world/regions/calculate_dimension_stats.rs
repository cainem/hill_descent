use std::cmp::Ordering;

/// Calculates the unique value count and standard deviation for each dimension.
///
/// # Arguments
/// * `expressed_values` - A slice of slices, where each inner slice represents an organism's
///   expressed genetic values for the problem dimensions.
/// * `num_dimensions` - The number of dimensions to analyze.
///
/// # Returns
/// A vector of tuples, where each tuple contains the unique value count (usize) and
/// the standard deviation (f64) for a dimension.
pub fn calculate_dimension_stats(
    expressed_values: &[&[f64]],
    num_dimensions: usize,
) -> Vec<(usize, f64)> {
    let mut dimension_stats = Vec::with_capacity(num_dimensions);
    let num_organisms = expressed_values.len();

    for i in 0..num_dimensions {
        let mut dimension_values = Vec::with_capacity(num_organisms);
        for values in expressed_values {
            if let Some(value) = values.get(i) {
                dimension_values.push(*value);
            }
        }

        let unique_count = {
            let mut sorted_values = dimension_values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            sorted_values.dedup();
            sorted_values.len()
        };

        let n = dimension_values.len();
        let std_dev = if n > 1 {
            let mean = dimension_values.iter().sum::<f64>() / (n as f64);
            let variance = dimension_values
                .iter()
                .map(|value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>()
                / ((n - 1) as f64);
            variance.sqrt()
        } else {
            0.0
        };

        dimension_stats.push((unique_count, std_dev));
    }
    dimension_stats
}

#[cfg(test)]
mod tests {
    use super::*;

    const F64_PRECISION: f64 = 1e-9;

    #[test]
    fn given_diverse_values_when_calculate_dimension_stats_then_returns_correct_stats() {
        let values: Vec<&[f64]> = vec![&[1.0, 10.0], &[1.0, 20.0], &[2.0, 30.0]];
        let stats = calculate_dimension_stats(&values, 2);

        assert_eq!(stats.len(), 2);
        // Dimension 0: [1.0, 1.0, 2.0] -> 2 unique, std_dev ~0.577
        assert_eq!(stats[0].0, 2);
        assert!((stats[0].1 - 0.577350269).abs() < F64_PRECISION);
        // Dimension 1: [10.0, 20.0, 30.0] -> 3 unique, std_dev 10.0
        assert_eq!(stats[1].0, 3);
        assert!((stats[1].1 - 10.0).abs() < F64_PRECISION);
    }

    #[test]
    fn given_no_diversity_when_calculate_dimension_stats_then_returns_zero_std_dev() {
        let values: Vec<&[f64]> = vec![&[5.0, 5.0], &[5.0, 5.0]];
        let stats = calculate_dimension_stats(&values, 2);

        assert_eq!(stats.len(), 2);
        // Dimension 0: 1 unique, std_dev 0.0
        assert_eq!(stats[0].0, 1);
        assert_eq!(stats[0].1, 0.0);
        // Dimension 1: 1 unique, std_dev 0.0
        assert_eq!(stats[1].0, 1);
        assert_eq!(stats[1].1, 0.0);
    }

    #[test]
    fn given_empty_values_when_calculate_dimension_stats_then_returns_empty_stats() {
        let values: Vec<&[f64]> = vec![];
        let stats = calculate_dimension_stats(&values, 0);
        assert!(stats.is_empty());
    }

    #[test]
    fn given_single_organism_when_calculate_dimension_stats_then_returns_zero_std_dev() {
        let values: Vec<&[f64]> = vec![&[10.0, 20.0]];
        let stats = calculate_dimension_stats(&values, 2);

        assert_eq!(stats.len(), 2);
        // Dimension 0: 1 unique, std_dev 0.0
        assert_eq!(stats[0].0, 1);
        assert_eq!(stats[0].1, 0.0);
        // Dimension 1: 1 unique, std_dev 0.0
        assert_eq!(stats[1].0, 1);
        assert_eq!(stats[1].1, 0.0);
    }
}
