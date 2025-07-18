// src/parameters/global_constants.rs

/// Holds global constants for the simulation that are not subject to evolutionary change.
#[derive(Debug, Clone, Copy)]
pub struct GlobalConstants {
    /// Total target population size (P).
    population_size: usize,
    /// Maximum number of regions the space can be divided into (Z).
    target_regions: usize,
}

impl GlobalConstants {
    /// Returns the total target population size.
    pub fn population_size(&self) -> usize {
        self.population_size
    }

    /// Returns the target number of regions.
    pub fn target_regions(&self) -> usize {
        self.target_regions
    }

    /// Creates a new instance of GlobalConstants.
    ///
    /// # Arguments
    ///
    /// * `population_size` - The total target population size.
    /// * `target_regions` - The maximum number of regions.
    ///
    /// # Panics
    ///
    /// Panics if `population_size` or `target_regions` is zero.
    pub fn new(population_size: usize, target_regions: usize) -> Self {
        if population_size == 0 {
            panic!("Population size cannot be zero.");
        }
        if target_regions == 0 {
            panic!("Max regions cannot be zero.");
        }

        Self {
            population_size,
            target_regions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_new_then_global_constants_is_created() {
        let population_size = 100;
        let target_regions = 1000;
        let constants = GlobalConstants::new(population_size, target_regions);

        assert_eq!(constants.population_size(), population_size);
        assert_eq!(constants.target_regions(), target_regions);
    }

    #[test]
    #[should_panic(expected = "Population size cannot be zero.")]
    fn given_zero_population_size_when_new_then_panics() {
        GlobalConstants::new(0, 1000);
    }

    #[test]
    #[should_panic(expected = "Max regions cannot be zero.")]
    fn given_zero_target_regions_when_new_then_panics() {
        GlobalConstants::new(100, 0);
    }
}
