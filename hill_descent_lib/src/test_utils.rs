use std::sync::Arc;

use crate::phenotype::Phenotype;
use crate::world::organisms::organism::Organism;

// This function is public within the crate for use by various tests.
pub(crate) fn create_test_organism() -> Arc<Organism> {
    let mut rng = rand::rng();
    // Phenotype requires at least NUM_SYSTEM_PARAMETERS bounds.
    let parameter_bounds: Vec<_> = (0..7).map(|_| 0.0..=1.0).collect();
    let phenotype = Arc::new(Phenotype::new_random_phenotype(&mut rng, &parameter_bounds));
    Arc::new(Organism::new(phenotype, 0, (None, None)))
}
