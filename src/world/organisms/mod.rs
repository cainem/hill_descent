pub mod generate_random_phenotypes;
pub mod new;

pub use generate_random_phenotypes::generate_random_phenotypes;

use crate::Phenotype;

#[derive(Debug, Clone)]
pub struct Organisms {
    _organisms: Vec<Phenotype>,
}
