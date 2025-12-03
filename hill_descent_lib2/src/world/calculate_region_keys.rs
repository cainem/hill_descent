//! Calculate region keys for all organisms.

use super::World;

impl World {
    /// Calculates region keys for all organisms.
    ///
    /// May loop if organisms are out of bounds, expanding dimensions as needed.
    ///
    /// # Returns
    ///
    /// The indices of dimensions that changed (empty if no expansion needed).
    pub fn calculate_region_keys(&mut self) -> Vec<usize> {
        todo!("Implement calculate_region_keys")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_organisms_in_bounds_when_calculate_then_returns_empty() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_organisms_out_of_bounds_when_calculate_then_expands_and_retries() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_expansion_when_calculate_then_dimension_version_increments() {
        todo!()
    }
}
