use crate::Phenotype;

#[derive(Debug, Clone)]
pub struct Region {
    _carrying_capacity: Option<usize>,
    _min_score: f64,
    _organisms: Vec<Phenotype>,
}
