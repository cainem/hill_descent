use super::Regions;

impl Regions {
    /// Sorts organisms within each region by fitness score (ascending) then age (descending).
    ///
    /// This implements the ranking rules from PDD §5.2.2:
    /// - Primary: Fitness score (lower is better)
    /// - Secondary: Age (older first for tie-breaking)
    /// - Any further tie is arbitrary
    ///
    /// This function should be called immediately after fitness evaluation and before
    /// any reproduction or population culling operations.
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self))
    )]
    pub fn sort_regions(&mut self) {
        for region in self.regions.values_mut() {
            // Sort organisms in-place by score (asc) then age (desc)
            region.organisms_mut().sort_by(|a, b| {
                let score_cmp = a
                    .score()
                    .unwrap_or(f64::INFINITY)
                    .partial_cmp(&b.score().unwrap_or(f64::INFINITY))
                    .unwrap_or(std::cmp::Ordering::Equal);
                score_cmp.then_with(|| b.age().cmp(&a.age()))
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parameters::global_constants::GlobalConstants,
        phenotype::Phenotype,
        world::{organisms::organism::Organism, regions::region::Region},
    };
    use std::rc::Rc;

    fn create_test_organism(score: f64, age: usize) -> Rc<Organism> {
        // Create phenotype with default system parameters + one problem parameter
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0, 0.5];
        let phenotype = Rc::new(Phenotype::new_for_test(expressed));
        let organism = Organism::new(phenotype, age, (None, None));
        organism.set_score(Some(score));
        Rc::new(organism)
    }

    #[test]
    fn given_unsorted_organisms_when_sort_regions_then_sorted_by_score_then_age() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        let region_key = vec![0];
        let mut region = Region::new();

        // Add organisms in unsorted order: (score, age)
        region.add_organism(create_test_organism(3.0, 5)); // worst score, middle age
        region.add_organism(create_test_organism(1.0, 3)); // best score, youngest
        region.add_organism(create_test_organism(2.0, 7)); // middle score, oldest
        region.add_organism(create_test_organism(2.0, 4)); // middle score, younger (tie-breaker)

        regions.regions.insert(region_key, region);

        // Act
        regions.sort_regions();

        // Assert
        let sorted_region = regions.regions.get(&vec![0]).unwrap();
        let organisms: Vec<_> = sorted_region.organisms().iter().collect();

        // Should be sorted by score (asc), then age (desc)
        assert_eq!(organisms[0].score().unwrap(), 1.0); // best score
        assert_eq!(organisms[0].age(), 3);

        assert_eq!(organisms[1].score().unwrap(), 2.0); // tied score, older first
        assert_eq!(organisms[1].age(), 7);

        assert_eq!(organisms[2].score().unwrap(), 2.0); // tied score, younger second
        assert_eq!(organisms[2].age(), 4);

        assert_eq!(organisms[3].score().unwrap(), 3.0); // worst score
        assert_eq!(organisms[3].age(), 5);
    }

    #[test]
    fn given_empty_regions_when_sort_regions_then_no_panic() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        // Act & Assert - should not panic
        regions.sort_regions();
    }

    #[test]
    fn given_single_organism_when_sort_regions_then_unchanged() {
        // Arrange
        let gc = GlobalConstants::new(10, 4);
        let mut regions = Regions::new(&gc);

        let region_key = vec![0];
        let mut region = Region::new();
        region.add_organism(create_test_organism(1.5, 10));
        regions.regions.insert(region_key, region);

        // Act
        regions.sort_regions();

        // Assert
        let sorted_region = regions.regions.get(&vec![0]).unwrap();
        let organisms: Vec<_> = sorted_region.organisms().iter().collect();
        assert_eq!(organisms.len(), 1);
        assert_eq!(organisms[0].score().unwrap(), 1.5);
        assert_eq!(organisms[0].age(), 10);
    }
}
