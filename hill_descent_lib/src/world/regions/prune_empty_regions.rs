impl super::Regions {
    /// Removes any regions that are no longer populated (i.e., contain no organisms).
    pub(super) fn prune_empty_regions(&mut self) {
        self.regions.retain(|_, region| !region.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use std::rc::Rc;

    use crate::parameters::global_constants::GlobalConstants;
    use crate::phenotype::Phenotype;
    use crate::world::organisms::organism::Organism;
    use crate::world::regions::Regions;
    use crate::world::regions::region::Region;

    // Helper to create a test organism (with phenotype) using Phenotype::new_for_test
    fn create_test_organism() -> Rc<Organism> {
        // Provide default system parameters as per MEMORY[f61e5e69-4a9e-4874-b29b-c77dd5f97ec4]
        // and MEMORY[0a820419-f45b-4e9d-8d4e-7c8901b664ed]
        let expressed_values = vec![
            0.1,   // m1_prob_false_to_true
            0.5,   // m2_prob_true_to_false
            0.001, // m3_prob_adj_double_halve_flag
            0.001, // m4_prob_adj_direction_flag
            0.001, // m5_prob_locus_value_mutation
            100.0, // max_age
            2.0,   // crossover_points
        ];
        let phenotype = Rc::new(Phenotype::new_for_test(expressed_values));
        Rc::new(Organism::new(Rc::clone(&phenotype), 0, (None, None)))
    }

    #[test]
    fn given_regions_with_some_empty_when_prune_empty_regions_then_empty_regions_are_removed() {
        let gc = GlobalConstants::new(100, 10); // population_size, target_regions
        let mut regions = Regions::new(&gc);
        let organism_rc = create_test_organism();

        let mut region1 = Region::new();
        region1.add_organism(Rc::clone(&organism_rc));

        let region2 = Region::new(); // Empty

        let mut region3 = Region::new();
        region3.add_organism(Rc::clone(&organism_rc));

        regions.regions =
            IndexMap::from_iter([(vec![0], region1), (vec![1], region2), (vec![2], region3)]);

        assert_eq!(regions.regions.len(), 3);
        regions.prune_empty_regions();
        assert_eq!(regions.regions.len(), 2);
        assert!(regions.regions.contains_key(&vec![0]));
        assert!(!regions.regions.contains_key(&vec![1])); // region2 should be removed
        assert!(regions.regions.contains_key(&vec![2]));
    }

    #[test]
    fn given_regions_with_all_populated_when_prune_empty_regions_then_no_regions_are_removed() {
        let gc = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&gc);
        let organism_rc = create_test_organism();

        let mut region1 = Region::new();
        region1.add_organism(Rc::clone(&organism_rc));
        let mut region2 = Region::new();
        region2.add_organism(Rc::clone(&organism_rc));

        regions.regions = IndexMap::from_iter([(vec![0], region1), (vec![1], region2)]);

        assert_eq!(regions.regions.len(), 2);
        regions.prune_empty_regions();
        assert_eq!(regions.regions.len(), 2);
    }

    #[test]
    fn given_regions_with_all_empty_when_prune_empty_regions_then_all_regions_are_removed() {
        let gc = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&gc);

        let region1 = Region::new(); // Empty
        let region2 = Region::new(); // Empty

        regions.regions = IndexMap::from_iter([(vec![0], region1), (vec![1], region2)]);

        assert_eq!(regions.regions.len(), 2);
        regions.prune_empty_regions();
        assert_eq!(regions.regions.len(), 0);
    }

    #[test]
    fn given_no_regions_when_prune_empty_regions_then_no_change_and_no_panic() {
        let gc = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&gc);
        assert_eq!(regions.regions.len(), 0);
        regions.prune_empty_regions();
        assert_eq!(regions.regions.len(), 0);
    }
}
