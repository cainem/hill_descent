use crate::Phenotype;

#[derive(Debug, Clone)]
pub struct Organism {
    _region_key: Option<Vec<usize>>,
    _score: Option<f64>,
    phenotype: Phenotype,
}

impl Organism {
    /// Creates a new `Organism` with the given phenotype.
    pub fn new(phenotype: Phenotype) -> Self {
        Self {
            _region_key: None,
            _score: None,
            phenotype,
        }
    }

    /// Returns a reference to the organism's phenotype.
    pub fn phenotype(&self) -> &Phenotype {
        &self.phenotype
    }

    /// Returns a mutable reference to the organism's phenotype.
    pub fn phenotype_mut(&mut self) -> &mut Phenotype {
        &mut self.phenotype
    }

    /// Returns the region key of the organism, if set.
    pub fn region_key(&self) -> Option<&Vec<usize>> {
        self._region_key.as_ref()
    }

    /// Sets the region key of the organism.
    pub fn set_region_key(&mut self, region_key: Option<Vec<usize>>) {
        self._region_key = region_key;
    }

    /// Returns the score of the organism, if set.
    pub fn score(&self) -> Option<f64> {
        self._score
    }

    /// Sets the score of the organism.
    pub fn set_score(&mut self, score: Option<f64>) {
        self._score = score;
    }
}
