use crate::world::organisms::Organisms;

impl super::Regions {
    /// Adds phenotypes from the given `Organisms` collection to their respective regions.
    ///
    /// Iterates through each organism in the `organisms` collection. If an organism
    /// has a region key, its phenotype (as an `Rc<Phenotype>`) is added to the
    /// corresponding `Region`. If a `Region` for a given key does not exist,
    /// it is created.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `Regions` instance.
    /// * `organisms` - A reference to the `Organisms` collection to process.
    pub fn add_phenotypes(&mut self, organisms: &Organisms) {
        for organism in organisms.iter() {
            if let Some(key) = organism.region_key() {
                let phenotype_rc = organism.get_phenotype_rc();
                let region = self.regions.entry(key.clone()).or_default();
                region.add_phenotype(phenotype_rc);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parameters::global_constants::GlobalConstants,
        phenotype::Phenotype,
        world::{organisms::Organisms, regions::Regions, world_function::WorldFunction},
    };
    use std::fmt;
    use std::rc::Rc;

    #[derive(Debug)]
    struct TestFn;
    impl WorldFunction for TestFn {
        fn run(&self, _p: &[f64]) -> Vec<f64> {
            vec![0.0]
        }
    }

    // Helper to create a Phenotype for testing
    // Phenotype::new_for_test requires at least NUM_SYSTEM_PARAMETERS (7) expressed values.
    fn mock_phenotype() -> Phenotype {
        let expressed_values: Vec<f64> = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]; // 7 values
        Phenotype::new_for_test(expressed_values)
    }

    #[test]
    fn given_empty_organisms_when_add_phenotypes_then_regions_unchanged() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let world_fn = Rc::new(TestFn);
        let organisms = Organisms::new_from_phenotypes(vec![], world_fn);

        regions.add_phenotypes(&organisms);

        assert_eq!(regions.regions().len(), 0);
    }

    #[test]
    fn given_organisms_with_no_region_keys_when_add_phenotypes_then_regions_unchanged() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);

        let phenotypes_for_organisms = vec![mock_phenotype()];
        let world_fn = Rc::new(TestFn);
        let organisms_collection =
            Organisms::new_from_phenotypes(phenotypes_for_organisms, world_fn);
        // Organisms created by new_from_phenotypes will have _region_key = None by default.

        regions.add_phenotypes(&organisms_collection);

        assert_eq!(regions.regions().len(), 0);
    }

    #[test]
    fn given_one_organism_with_region_key_when_add_phenotypes_then_region_created_with_phenotype() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key1 = vec![1, 2, 3];

        let world_fn = Rc::new(TestFn);
        let mut organisms_collection =
            Organisms::new_from_phenotypes(vec![mock_phenotype()], world_fn);
        let phenotype_rc_from_org = organisms_collection
            .iter()
            .next()
            .unwrap()
            .get_phenotype_rc();
        organisms_collection
            .iter_mut()
            .next()
            .unwrap()
            .set_region_key(Some(region_key1.clone()));

        regions.add_phenotypes(&organisms_collection);

        assert_eq!(regions.regions().len(), 1);
        let region = regions
            .get_region(&region_key1)
            .expect("Region should exist");
        assert_eq!(region.organism_count(), 1);
        assert!(Rc::ptr_eq(
            &region.get_organisms()[0],
            &phenotype_rc_from_org
        ));
    }

    #[test]
    fn given_multiple_organisms_same_key_when_add_phenotypes_then_region_has_all_phenotypes() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key = vec![1];

        let world_fn = Rc::new(TestFn);
        let mut organisms_collection =
            Organisms::new_from_phenotypes(vec![mock_phenotype(), mock_phenotype()], world_fn);
        let mut org_iter_mut = organisms_collection.iter_mut();

        let org1_mut = org_iter_mut.next().unwrap();
        org1_mut.set_region_key(Some(region_key.clone()));
        let pheno1_rc_from_org = org1_mut.get_phenotype_rc();

        let org2_mut = org_iter_mut.next().unwrap();
        org2_mut.set_region_key(Some(region_key.clone()));
        let pheno2_rc_from_org = org2_mut.get_phenotype_rc();

        regions.add_phenotypes(&organisms_collection);

        assert_eq!(regions.regions().len(), 1);
        let region = regions
            .get_region(&region_key)
            .expect("Region should exist");
        assert_eq!(region.organism_count(), 2);

        let region_phenotypes = region.get_organisms();
        assert!(
            region_phenotypes
                .iter()
                .any(|p| Rc::ptr_eq(p, &pheno1_rc_from_org))
        );
        assert!(
            region_phenotypes
                .iter()
                .any(|p| Rc::ptr_eq(p, &pheno2_rc_from_org))
        );
    }

    #[test]
    fn given_multiple_organisms_different_keys_when_add_phenotypes_then_regions_created_correctly()
    {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key1 = vec![1];
        let region_key2 = vec![2];

        let world_fn = Rc::new(TestFn);
        let mut organisms_collection =
            Organisms::new_from_phenotypes(vec![mock_phenotype(), mock_phenotype()], world_fn);
        let mut iter_mut = organisms_collection.iter_mut();

        let organism1_mut = iter_mut.next().unwrap();
        organism1_mut.set_region_key(Some(region_key1.clone()));
        let pheno1_rc_from_org = organism1_mut.get_phenotype_rc();

        let organism2_mut = iter_mut.next().unwrap();
        organism2_mut.set_region_key(Some(region_key2.clone()));
        let pheno2_rc_from_org = organism2_mut.get_phenotype_rc();

        regions.add_phenotypes(&organisms_collection);

        assert_eq!(regions.regions().len(), 2);

        let region1 = regions
            .get_region(&region_key1)
            .expect("Region 1 should exist");
        assert_eq!(region1.organism_count(), 1);
        assert!(Rc::ptr_eq(&region1.get_organisms()[0], &pheno1_rc_from_org));

        let region2 = regions
            .get_region(&region_key2)
            .expect("Region 2 should exist");
        assert_eq!(region2.organism_count(), 1);
        assert!(Rc::ptr_eq(&region2.get_organisms()[0], &pheno2_rc_from_org));
    }

    #[test]
    fn given_region_with_existing_phenotype_when_add_more_phenotypes_to_same_key_then_all_are_present()
     {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key = vec![1, 0, 0];

        let world_fn = Rc::new(TestFn);

        // First, add one organism to create the region and put one phenotype in it
        let mut initial_organisms =
            Organisms::new_from_phenotypes(vec![mock_phenotype()], world_fn.clone());
        let existing_phenotype_rc = initial_organisms.iter().next().unwrap().get_phenotype_rc();
        initial_organisms
            .iter_mut()
            .next()
            .unwrap()
            .set_region_key(Some(region_key.clone()));
        regions.add_phenotypes(&initial_organisms);

        // Now, prepare a new organism to be added to the same region
        let mut new_organisms_to_add =
            Organisms::new_from_phenotypes(vec![mock_phenotype()], world_fn.clone());
        let new_phenotype_rc = new_organisms_to_add
            .iter()
            .next()
            .unwrap()
            .get_phenotype_rc();
        new_organisms_to_add
            .iter_mut()
            .next()
            .unwrap()
            .set_region_key(Some(region_key.clone()));

        // Act: add the new organism
        regions.add_phenotypes(&new_organisms_to_add);

        // Assert
        assert_eq!(regions.regions().len(), 1); // Still only one region
        let region = regions
            .get_region(&region_key)
            .expect("Region should exist");
        assert_eq!(region.organism_count(), 2); // Should now have two phenotypes

        let region_phenotypes = region.get_organisms();
        assert!(
            region_phenotypes
                .iter()
                .any(|p| Rc::ptr_eq(p, &existing_phenotype_rc))
        );
        assert!(
            region_phenotypes
                .iter()
                .any(|p| Rc::ptr_eq(p, &new_phenotype_rc))
        );
    }
}
