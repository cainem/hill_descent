// src/parameters/global_constants.rs

/// Holds global constants for the simulation that are not subject to evolutionary change.
#[derive(Debug, Clone, Copy)]
pub struct GlobalConstants {
    /// Total target population size (P).
    population_size: usize,
    /// Maximum number of regions the space can be divided into (Z).
    max_regions: usize,
}

impl GlobalConstants {
    /// Returns the total target population size.
    pub fn population_size(&self) -> usize {
        self.population_size
    }

    /// Returns the maximum number of regions.
    pub fn max_regions(&self) -> usize {
        self.max_regions
    }

    /// Creates a new instance of GlobalConstants.
    ///
    /// # Arguments
    ///
    /// * `population_size` - The total target population size.
    /// * `max_regions` - The maximum number of regions.
    ///
    /// # Panics
    ///
    /// Panics if `population_size` or `max_regions` is zero.
    pub fn new(population_size: usize, max_regions: usize) -> Self {
        if population_size == 0 {
            panic!("Population size cannot be zero.");
        }
        if max_regions == 0 {
            panic!("Max regions cannot be zero.");
        }

        Self {
            population_size,
            max_regions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_new_then_global_constants_is_created() {
        let population_size = 100;
        let max_regions = 1000;
        let constants = GlobalConstants::new(population_size, max_regions);

        assert_eq!(constants.population_size(), population_size);
        assert_eq!(constants.max_regions(), max_regions);
    }

    #[test]
    #[should_panic(expected = "Population size cannot be zero.")]
    fn given_zero_population_size_when_new_then_panics() {
        GlobalConstants::new(0, 1000);
    }

    #[test]
    #[should_panic(expected = "Max regions cannot be zero.")]
    fn given_zero_max_regions_when_new_then_panics() {
        GlobalConstants::new(100, 0);
    }
}
