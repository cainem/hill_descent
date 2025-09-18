#[cfg(test)]
use crate::phenotype::Phenotype;
use crate::world::organisms::organism::Organism;
use std::rc::Rc;

fn create_test_phenotype() -> Phenotype {
        // Create a minimal phenotype with 7 system parameters
        Phenotype::new_for_test(vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0])
    }

    #[test]
    fn given_new_organism_when_created_with_new_then_is_root() {
        let phenotype = Rc::new(create_test_phenotype());
        let organism = Organism::new(phenotype, 0);

        assert!(organism.is_root());
        assert_eq!(organism.parent_count(), 0);
        assert_eq!(organism.parent_ids(), (None, None));
    }

    #[test]
    fn given_new_organism_when_created_with_asexual_parents_then_has_one_parent() {
        let phenotype = Rc::new(create_test_phenotype());
        let parent_id = 42;
        let organism = Organism::new_with_parents(phenotype, 0, (Some(parent_id), None));

        assert!(!organism.is_root());
        assert_eq!(organism.parent_count(), 1);
        assert_eq!(organism.parent_ids(), (Some(parent_id), None));
    }

    #[test]
    fn given_new_organism_when_created_with_sexual_parents_then_has_two_parents() {
        let phenotype = Rc::new(create_test_phenotype());
        let parent1_id = 42;
        let parent2_id = 84;
        let organism =
            Organism::new_with_parents(phenotype, 0, (Some(parent1_id), Some(parent2_id)));

        assert!(!organism.is_root());
        assert_eq!(organism.parent_count(), 2);
        assert_eq!(organism.parent_ids(), (Some(parent1_id), Some(parent2_id)));
    }

    #[test]
    fn given_new_organism_when_created_with_root_parents_then_is_root() {
        let phenotype = Rc::new(create_test_phenotype());
        let organism = Organism::new_with_parents(phenotype, 0, (None, None));

        assert!(organism.is_root());
        assert_eq!(organism.parent_count(), 0);
        assert_eq!(organism.parent_ids(), (None, None));
    }
