impl super::Regions {
    pub fn get_most_common_key(&self) -> Option<Vec<usize>> {
        self.regions()
            .iter()
            .max_by_key(|(_, region)| region.organism_count())
            .map(|(key, _)| key.clone())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::parameters::global_constants::GlobalConstants;
    use crate::test_utils::create_test_organism;
    use crate::world::regions::Regions;
    use crate::world::regions::region::Region;

    #[test]
    fn given_no_regions_when_get_most_common_key_then_returns_none() {
        let global_constants = GlobalConstants::new(100, 10);
        let regions = Regions::new(&global_constants);
        assert!(regions.get_most_common_key().is_none());
    }

    #[test]
    fn given_one_region_when_get_most_common_key_then_returns_its_key() {
        let global_constants = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&global_constants);
        let key = vec![1];
        regions.regions_mut().insert(key.clone(), Region::new());
        assert_eq!(regions.get_most_common_key(), Some(key));
    }

    #[test]
    fn given_multiple_regions_when_get_most_common_key_then_returns_correct_key() {
        let global_constants = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&global_constants);

        let key1 = vec![1];
        let mut region1 = Region::new();
        region1.add_organism(create_test_organism());

        let key2 = vec![2];
        let mut region2 = Region::new();
        region2.add_organism(create_test_organism());
        region2.add_organism(create_test_organism()); // most common

        let key3 = vec![3];
        let region3 = Region::new(); // empty

        regions.regions_mut().insert(key1, region1);
        regions.regions_mut().insert(key2.clone(), region2);
        regions.regions_mut().insert(key3, region3);

        assert_eq!(regions.get_most_common_key(), Some(key2));
    }

    #[test]
    fn given_regions_with_a_tie_when_get_most_common_key_then_returns_one_of_them() {
        let global_constants = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&global_constants);

        let key1 = vec![1];
        let mut region1 = Region::new();
        region1.add_organism(create_test_organism());

        let key2 = vec![2];
        let mut region2 = Region::new();
        region2.add_organism(create_test_organism());

        regions.regions_mut().insert(key1.clone(), region1);
        regions.regions_mut().insert(key2.clone(), region2);

        // BTreeMap iterates keys in sorted order, so max_by_key will find key1 first in a tie.
        // If the implementation changes, this test might need adjustment.
        let most_common_key = regions.get_most_common_key();
        assert!(most_common_key == Some(key1) || most_common_key == Some(key2));
    }

    #[test]
    fn given_all_regions_are_empty_when_get_most_common_key_then_returns_a_key() {
        let global_constants = GlobalConstants::new(100, 10);
        let mut regions = Regions::new(&global_constants);

        let key1 = vec![1];
        let key2 = vec![2];

        regions.regions_mut().insert(key1.clone(), Region::new());
        regions.regions_mut().insert(key2.clone(), Region::new());

        let most_common_key = regions.get_most_common_key();
        assert!(most_common_key.is_some());
        // The specific key depends on BTreeMap iteration order.
        assert!(most_common_key == Some(key1) || most_common_key == Some(key2));
    }
}
