use serde::Serialize;
use serde_json;

use super::dimensions::dimension::Dimension;
use super::organisms::organism::Organism;
use super::regions::region::Region;

// Helper structs purely for serialisation of the World state -----------------
#[derive(Serialize)]
struct DimensionState {
    range: (f64, f64),
    number_of_divisions: usize,
}

#[derive(Serialize)]
struct PhenotypeState {
    expressed_values: Vec<f64>,
}

#[derive(Serialize)]
struct OrganismState {
    region_key: Option<Vec<usize>>,
    age: usize,
    score: Option<f64>,
    is_dead: bool,
    phenotype: PhenotypeState,
}

#[derive(Serialize)]
struct RegionState {
    key: Vec<usize>,
    min_score: Option<f64>,
    carrying_capacity: Option<usize>,
    organism_count: usize,
}

#[derive(Serialize)]
struct WorldState {
    dimensions: Vec<DimensionState>,
    organisms: Vec<OrganismState>,
    regions: Vec<RegionState>,
}

impl DimensionState {
    fn from_dimension(d: &Dimension) -> Self {
        let range = (*d.range().start(), *d.range().end());
        Self {
            range,
            number_of_divisions: d.number_of_divisions(),
        }
    }
}

impl PhenotypeState {
    fn from_phenotype(p: &crate::phenotype::Phenotype) -> Self {
        Self {
            expressed_values: p.expressed_values().to_vec(),
        }
    }
}

impl OrganismState {
    fn from_organism(o: &Organism) -> Self {
        Self {
            region_key: o.region_key().cloned(),
            age: o.age(),
            score: o.score(),
            is_dead: o.is_dead(),
            phenotype: PhenotypeState::from_phenotype(o.phenotype()),
        }
    }
}

impl RegionState {
    fn from_region(key: &[usize], r: &Region) -> Self {
        Self {
            key: key.to_vec(),
            min_score: r.min_score(),
            carrying_capacity: r.carrying_capacity(),
            organism_count: r.organism_count(),
        }
    }
}

impl super::World {
    /// Returns a `String` containing a JSON representation of the current World state.
    ///
    /// Only the `dimensions`, `organisms` and `regions` fields are represented, as requested.
    pub fn get_state(&self) -> String {
        let dimensions: Vec<DimensionState> = self
            .dimensions
            .get_dimensions()
            .iter()
            .map(DimensionState::from_dimension)
            .collect();

        let organisms: Vec<OrganismState> = self
            .organisms
            .iter()
            .map(OrganismState::from_organism)
            .collect();

        let regions: Vec<RegionState> = self
            .regions
            .regions()
            .iter()
            .map(|(k, r)| RegionState::from_region(k, r))
            .collect();

        let state = WorldState {
            dimensions,
            organisms,
            regions,
        };

        serde_json::to_string(&state).expect("Serialization of World state failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::global_constants::GlobalConstants;
    use crate::world::world_function::WorldFunction;
    use std::ops::RangeInclusive;

    // Minimal WorldFunction for tests
    #[derive(Debug)]
    struct DummyFn;
    impl WorldFunction for DummyFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![0.0]
        }
    }

    #[test]
    fn given_world_when_get_state_then_returns_valid_json() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, 10.0..=11.0];
        let gc = GlobalConstants::new(4, 10);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        // Attempt to parse to ensure it is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("dimensions").is_some());
        assert!(parsed.get("organisms").is_some());
        assert!(parsed.get("regions").is_some());
    }
}
