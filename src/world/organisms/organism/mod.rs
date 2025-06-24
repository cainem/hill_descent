use std::rc::Rc;

use crate::phenotype::Phenotype;

pub mod update_region_key;

#[derive(Debug, Clone)]
pub struct Organism {
    region_key: Option<Vec<usize>>,
    score: Option<f64>,
    phenotype: Rc<Phenotype>,
}

impl Organism {
    /// Creates a new `Organism` with the given phenotype.
    pub fn new(phenotype: Rc<Phenotype>) -> Self {
        Self {
            region_key: None,
            score: None,
            phenotype,
        }
    }

    /// Returns a reference to the organism's phenotype.
    pub fn phenotype(&self) -> &Phenotype {
        &self.phenotype
    }

    /// Returns a clone of the organism's phenotype Rc.
    pub fn get_phenotype_rc(&self) -> Rc<Phenotype> {
        Rc::clone(&self.phenotype)
    }

    /// Returns the region key of the organism, if set.
    pub fn region_key(&self) -> Option<&Vec<usize>> {
        self.region_key.as_ref()
    }

    /// Sets the region key of the organism.
    pub fn set_region_key(&mut self, region_key: Option<Vec<usize>>) {
        self.region_key = region_key;
    }

    /// Returns the score of the organism, if set.
    pub fn score(&self) -> Option<f64> {
        self.score
    }

    /// Sets the score of the organism.
    pub fn set_score(&mut self, score: Option<f64>) {
        self.score = score;
    }
}
