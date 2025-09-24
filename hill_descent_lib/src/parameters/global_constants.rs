// src/parameters/global_constants.rs

/// Holds global constants for the simulation that are not subject to evolutionary change.
#[derive(Debug, Clone, Copy)]
pub struct GlobalConstants {
    /// Total target population size (P).
    population_size: usize,
    /// Maximum number of regions the space can be divided into (Z).
    target_regions: usize,
    /// Seed for the world's random number generator (for reproducible simulations).
    world_seed: u64,
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

    /// Returns the world seed for the random number generator.
    pub fn world_seed(&self) -> u64 {
        self.world_seed
    }

    /// Creates a new instance of GlobalConstants with default world seed.
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
        const DEFAULT_WORLD_SEED: u64 = 2_147_483_647; // A Mersenne prime (2^31 - 1)
        Self::new_with_seed(population_size, target_regions, DEFAULT_WORLD_SEED)
    }

    /// Creates a new instance of GlobalConstants with custom world seed.
    ///
    /// # Arguments
    ///
    /// * `population_size` - The total target population size.
    /// * `target_regions` - The maximum number of regions.
    /// * `world_seed` - The seed for the world's random number generator.
    ///
    /// # Panics
    ///
    /// Panics if `population_size` or `target_regions` is zero.
    pub fn new_with_seed(population_size: usize, target_regions: usize, world_seed: u64) -> Self {
        if population_size == 0 {
            panic!("Population size cannot be zero.");
        }
        if target_regions == 0 {
            panic!("Max regions cannot be zero.");
        }
        if target_regions > population_size {
            panic!("population size must be greater than target regions")
        }

        Self {
            population_size,
            target_regions,
            world_seed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_inputs_when_new_then_global_constants_is_created() {
        let population_size = 100;
        let target_regions = 10;
        let constants = GlobalConstants::new(population_size, target_regions);

        assert_eq!(constants.population_size(), population_size);
        assert_eq!(constants.target_regions(), target_regions);
        assert_eq!(constants.world_seed(), 2_147_483_647); // Default seed
    }

    #[test]
    fn given_valid_inputs_with_custom_seed_when_new_with_seed_then_global_constants_is_created() {
        let population_size = 100;
        let target_regions = 10;
        let world_seed = 12345;
        let constants = GlobalConstants::new_with_seed(population_size, target_regions, world_seed);

        assert_eq!(constants.population_size(), population_size);
        assert_eq!(constants.target_regions(), target_regions);
        assert_eq!(constants.world_seed(), world_seed);
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
