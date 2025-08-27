use crate::world::organisms::Organisms;
use crate::world::organisms::organism::Organism;
use std::rc::Rc;

impl super::Regions {
    /// Adds organisms from the given `Organisms` collection to their respective regions.
    ///
    /// Iterates through each organism in the `organisms` collection. Each organism
    /// must have a region key, and its organism is added to the
    /// corresponding `Region`. If a `Region` for a given key does not exist,
    /// it is created.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the `Regions` instance.
    /// * `organisms` - A reference to the `Organisms` collection to process.
    ///
    /// # Panics
    ///
    /// Panics if any organism does not have a region key, as this indicates
    /// a serious bug in the system.
    pub fn add_organisms(&mut self, organisms: &Organisms) {
        for organism in organisms.iter() {
            let key = organism
                .region_key()
                .expect("All organisms must have a region key when adding to regions");
            let organism_rc: Rc<Organism> = Rc::clone(organism);
            let region = self.regions.entry(key.clone()).or_default();
            region.add_organism(organism_rc);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parameters::global_constants::GlobalConstants,
        phenotype::Phenotype,
        world::{organisms::Organisms, regions::Regions},
    };
    use std::rc::Rc;

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
        let organisms = Organisms::new_from_phenotypes(vec![]);

        regions.add_organisms(&organisms);

        assert_eq!(regions.len(), 0);
    }

    #[test]
    #[should_panic(expected = "All organisms must have a region key when adding to regions")]
    fn given_organisms_with_no_region_keys_when_add_organisms_then_panics() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);

        let orgtypes_for_organisms = vec![mock_phenotype()];
        let organisms_collection = Organisms::new_from_phenotypes(orgtypes_for_organisms);
        // Organisms created by new_from_phenotypes will have _region_key = None by default.

        // This should panic because organisms don't have region keys
        regions.add_organisms(&organisms_collection);
    }

    #[test]
    fn given_one_organism_with_region_key_when_add_phenotypes_then_region_created_with_orgtype() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key1 = vec![1, 2, 3];

        let organisms_collection = Organisms::new_from_phenotypes(vec![mock_phenotype()]);
        let orgtype_rc_from_org = organisms_collection
            .iter()
            .next()
            .unwrap()
            .get_phenotype_rc();
        organisms_collection
            .iter()
            .next()
            .unwrap()
            .set_region_key(Some(region_key1.clone()));

        regions.add_organisms(&organisms_collection);

        assert_eq!(regions.len(), 1);
        let region = regions
            .get_region(&region_key1)
            .expect("Region should exist");
        assert_eq!(region.organism_count(), 1);
        assert!(Rc::ptr_eq(
            &region.organisms()[0].get_phenotype_rc(),
            &orgtype_rc_from_org
        ));
    }

    #[test]
    fn given_multiple_organisms_same_key_when_add_phenotypes_then_region_has_all_orgtypes() {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key = vec![1];

        let organisms_collection =
            Organisms::new_from_phenotypes(vec![mock_phenotype(), mock_phenotype()]);
        let mut org_iter_mut = organisms_collection.iter();

        let org1_mut = org_iter_mut.next().unwrap();
        org1_mut.set_region_key(Some(region_key.clone()));
        let org1_rc_from_org = org1_mut.get_phenotype_rc();

        let org2_mut = org_iter_mut.next().unwrap();
        org2_mut.set_region_key(Some(region_key.clone()));
        let org2_rc_from_org = org2_mut.get_phenotype_rc();

        regions.add_organisms(&organisms_collection);

        assert_eq!(regions.len(), 1);
        let region = regions
            .get_region(&region_key)
            .expect("Region should exist");
        assert_eq!(region.organism_count(), 2);

        let region_orgtypes = region.organisms();
        assert!(
            region_orgtypes
                .iter()
                .any(|p| Rc::ptr_eq(&p.get_phenotype_rc(), &org1_rc_from_org))
        );
        assert!(
            region_orgtypes
                .iter()
                .any(|p| Rc::ptr_eq(&p.get_phenotype_rc(), &org2_rc_from_org))
        );
    }

    #[test]
    fn given_multiple_organisms_different_keys_when_add_phenotypes_then_regions_created_correctly()
    {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key1 = vec![1];
        let region_key2 = vec![2];

        let organisms_collection =
            Organisms::new_from_phenotypes(vec![mock_phenotype(), mock_phenotype()]);
        let mut iter_mut = organisms_collection.iter();

        let organism1_mut = iter_mut.next().unwrap();
        organism1_mut.set_region_key(Some(region_key1.clone()));
        let org1_rc_from_org = organism1_mut.get_phenotype_rc();

        let organism2_mut = iter_mut.next().unwrap();
        organism2_mut.set_region_key(Some(region_key2.clone()));
        let org2_rc_from_org = organism2_mut.get_phenotype_rc();

        regions.add_organisms(&organisms_collection);

        assert_eq!(regions.len(), 2);

        let region1 = regions
            .get_region(&region_key1)
            .expect("Region 1 should exist");
        assert_eq!(region1.organism_count(), 1);
        assert!(Rc::ptr_eq(
            &region1.organisms()[0].get_phenotype_rc(),
            &org1_rc_from_org
        ));

        let region2 = regions
            .get_region(&region_key2)
            .expect("Region 2 should exist");
        assert_eq!(region2.organism_count(), 1);
        assert!(Rc::ptr_eq(
            &region2.organisms()[0].get_phenotype_rc(),
            &org2_rc_from_org
        ));
    }

    #[test]
    fn given_region_with_existing_orgtype_when_add_more_orgtypes_to_same_key_then_all_are_present()
    {
        let global_constants = GlobalConstants::new(10, 10);
        let mut regions = Regions::new(&global_constants);
        let region_key = vec![1, 0, 0];

        // First, add one organism to create the region and put one orgtype in it
        let initial_organisms = Organisms::new_from_phenotypes(vec![mock_phenotype()]);
        let existing_orgtype_rc = initial_organisms.iter().next().unwrap().get_phenotype_rc();
        initial_organisms
            .iter()
            .next()
            .unwrap()
            .set_region_key(Some(region_key.clone()));
        regions.add_organisms(&initial_organisms);

        // Now, prepare a new organism to be added to the same region
        let new_organisms_to_add = Organisms::new_from_phenotypes(vec![mock_phenotype()]);
        let new_orgtype_rc = new_organisms_to_add
            .iter()
            .next()
            .unwrap()
            .get_phenotype_rc();
        new_organisms_to_add
            .iter()
            .next()
            .unwrap()
            .set_region_key(Some(region_key.clone()));

        // Act: add the new organism
        regions.add_organisms(&new_organisms_to_add);

        // Assert
        assert_eq!(regions.len(), 1); // Still only one region
        let region = regions
            .get_region(&region_key)
            .expect("Region should exist");
        assert_eq!(region.organism_count(), 2); // Should now have two orgtypes

        // Check that both orgtypes are in the region
        let region_orgtypes = region.organisms();
        assert!(
            region_orgtypes
                .iter()
                .any(|p| Rc::ptr_eq(&p.get_phenotype_rc(), &existing_orgtype_rc))
        );
        assert!(
            region_orgtypes
                .iter()
                .any(|p| Rc::ptr_eq(&p.get_phenotype_rc(), &new_orgtype_rc))
        );
    }
}
