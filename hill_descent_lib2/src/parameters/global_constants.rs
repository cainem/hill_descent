// src/parameters/global_constants.rs

/// Configuration parameters for the genetic algorithm optimization.
///
/// This struct holds the core settings that control the behavior of the genetic algorithm,
/// including population size, spatial partitioning, and random seed for reproducibility.
///
/// # Examples
///
/// ```rust
/// use hill_descent_lib2::GlobalConstants;
///
/// // Create with default seed
/// let constants = GlobalConstants::new(100, 10);
/// assert_eq!(constants.population_size(), 100);
/// assert_eq!(constants.target_regions(), 10);
///
/// // Create with custom seed for reproducible results
/// let constants = GlobalConstants::new_with_seed(500, 25, 12345);
/// assert_eq!(constants.world_seed(), 12345);
/// ```
///
/// # Population Size
///
/// The population size determines how many organisms (candidate solutions) exist simultaneously.
/// Larger populations explore more of the search space but require more computation per generation.
///
/// **Guidelines:**
/// - Small problems (2-5 dimensions): 50-200
/// - Medium problems (6-20 dimensions): 200-1000
/// - Large problems (20+ dimensions): 1000-10000
///
/// # Target Regions
///
/// The search space is divided into spatial regions that adapt based on organism distribution.
/// More regions allow finer-grained local competition but increase overhead.
///
/// **Guidelines:**
/// - Use 5-20% of population size (e.g., pop=100 → regions=10)
/// - More dimensions may benefit from more regions
/// - Fewer regions encourage global exploration
/// - More regions encourage local exploitation
///
/// # Determinism
///
/// The `world_seed` ensures reproducible results. The same seed with the same configuration
/// will produce identical optimization runs, which is valuable for debugging and comparison.
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
    ///
    /// This is the number of organisms (candidate solutions) maintained during optimization.
    pub fn population_size(&self) -> usize {
        self.population_size
    }

    /// Returns the target number of regions.
    ///
    /// The search space is divided into this many adaptive regions for localized competition.
    pub fn target_regions(&self) -> usize {
        self.target_regions
    }

    /// Returns the world seed for the random number generator.
    ///
    /// Using the same seed ensures reproducible optimization runs.
    pub fn world_seed(&self) -> u64 {
        self.world_seed
    }

    /// Creates a new instance of `GlobalConstants` with default world seed.
    ///
    /// The default seed is 2,147,483,647 (a Mersenne prime: 2³¹ - 1).
    ///
    /// # Arguments
    ///
    /// * `population_size` - The total target population size. Must be > 0.
    /// * `target_regions` - The maximum number of regions. Must be > 0 and ≤ population_size.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `population_size` is zero
    /// - `target_regions` is zero
    /// - `target_regions` > `population_size`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hill_descent_lib2::GlobalConstants;
    ///
    /// let constants = GlobalConstants::new(100, 10);
    /// assert_eq!(constants.population_size(), 100);
    /// assert_eq!(constants.target_regions(), 10);
    /// ```
    pub fn new(population_size: usize, target_regions: usize) -> Self {
        const DEFAULT_WORLD_SEED: u64 = 2_147_483_647; // A Mersenne prime (2^31 - 1)
        Self::new_with_seed(population_size, target_regions, DEFAULT_WORLD_SEED)
    }

    /// Creates a new instance of `GlobalConstants` with custom world seed.
    ///
    /// Use this when you need reproducible results with a specific seed, such as for
    /// testing or comparing different optimization strategies.
    ///
    /// # Arguments
    ///
    /// * `population_size` - The total target population size. Must be > 0.
    /// * `target_regions` - The maximum number of regions. Must be > 0 and ≤ population_size.
    /// * `world_seed` - The seed for the world's random number generator.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `population_size` is zero
    /// - `target_regions` is zero
    /// - `target_regions` > `population_size`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hill_descent_lib2::GlobalConstants;
    ///
    /// // Reproducible optimization with custom seed
    /// let constants1 = GlobalConstants::new_with_seed(100, 10, 42);
    /// let constants2 = GlobalConstants::new_with_seed(100, 10, 42);
    /// assert_eq!(constants1.world_seed(), constants2.world_seed());
    /// ```
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
