//! Region key calculation from phenotype values and dimensions.

use super::Dimensions;
use crate::world::regions::region_key::RegionKey;

/// Result of calculating a dimensions key (region key).
#[derive(Debug, Clone)]
pub enum CalculateDimensionsKeyResult {
    /// Key calculated successfully
    Ok(RegionKey),
    /// Value is outside dimension bounds
    OutOfBounds {
        /// Indices of dimensions where values exceed bounds
        dimensions_exceeded: Vec<usize>,
    },
}

/// Calculates the region key for given phenotype values.
///
/// # Arguments
///
/// * `expressed_values` - The expressed phenotype values (excluding system parameters)
/// * `dimensions` - The current dimensions
///
/// # Returns
///
/// * `Ok(RegionKey)` - The calculated region key
/// * `OutOfBounds` - If any value exceeds dimension bounds
pub fn calculate_dimensions_key(
    expressed_values: &[f64],
    dimensions: &Dimensions,
) -> CalculateDimensionsKeyResult {
    todo!("Implement calculate_dimensions_key")
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_values_within_bounds_when_calculate_then_returns_ok() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_value_below_min_when_calculate_then_returns_out_of_bounds() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_value_above_max_when_calculate_then_returns_out_of_bounds() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_multiple_out_of_bounds_when_calculate_then_returns_all_exceeded() {
        todo!()
    }
}
