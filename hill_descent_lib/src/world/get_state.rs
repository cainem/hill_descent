use serde::Serialize;
use serde_json;

use super::dimensions::dimension::Dimension;
use super::organisms::organism::Organism;
use super::regions::region::Region;

// Helper structs purely for serialisation of the World state -----------------
#[derive(Serialize)]
struct DimensionState {
    range: (f64, f64),
    number_of_doublings: usize,
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
            number_of_doublings: d.number_of_doublings(),
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
            region_key: o.region_key(),
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
            .map(|rc| OrganismState::from_organism(rc.as_ref()))
            .collect();

        let regions: Vec<RegionState> = self
            .regions
            .iter_regions()
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
        let gc = GlobalConstants::new(10, 4);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        // Attempt to parse to ensure it is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("dimensions").is_some());
        assert!(parsed.get("organisms").is_some());
        assert!(parsed.get("regions").is_some());
    }

    #[test]
    fn given_world_when_get_state_then_json_contains_correct_dimension_data() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0, -5.0..=5.0];
        let gc = GlobalConstants::new(5, 2);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let dimensions = parsed.get("dimensions").unwrap().as_array().unwrap();
        assert_eq!(dimensions.len(), 2);

        // Check first dimension - bounds may have been adjusted
        let dim0 = &dimensions[0];
        assert!(dim0.get("range").is_some());
        assert!(dim0.get("number_of_doublings").is_some());
        assert_eq!(
            dim0.get("number_of_doublings").unwrap().as_u64().unwrap(),
            0
        );

        // Check second dimension - bounds may have been adjusted
        let dim1 = &dimensions[1];
        assert!(dim1.get("range").is_some());
        assert!(dim1.get("number_of_doublings").is_some());
    }

    #[test]
    fn given_empty_world_when_get_state_then_json_is_still_valid() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(1, 1); // At least 1 organism required
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("dimensions").is_some());
        assert!(parsed.get("organisms").is_some());
        assert!(parsed.get("regions").is_some());

        let organisms = parsed.get("organisms").unwrap().as_array().unwrap();
        assert_eq!(organisms.len(), 1);
    }

    #[test]
    fn given_world_when_get_state_then_json_contains_organism_data() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(3, 2);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let organisms = parsed.get("organisms").unwrap().as_array().unwrap();
        assert_eq!(organisms.len(), 3);

        // Check that each organism has expected fields
        for org in organisms {
            assert!(org.get("region_key").is_some());
            assert!(org.get("age").is_some());
            assert!(org.get("score").is_some());
            assert!(org.get("is_dead").is_some());
            assert!(org.get("phenotype").is_some());

            let phenotype = org.get("phenotype").unwrap();
            assert!(phenotype.get("expressed_values").is_some());
            let expressed_values = phenotype
                .get("expressed_values")
                .unwrap()
                .as_array()
                .unwrap();
            assert!(!expressed_values.is_empty());
        }
    }

    #[test]
    fn given_world_when_get_state_then_json_contains_region_data() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(10, 4);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let regions = parsed.get("regions").unwrap().as_array().unwrap();
        assert!(!regions.is_empty());

        // Check that each region has expected fields
        for region in regions {
            assert!(region.get("key").is_some());
            assert!(region.get("min_score").is_some());
            assert!(region.get("carrying_capacity").is_some());
            assert!(region.get("organism_count").is_some());

            let key = region.get("key").unwrap().as_array().unwrap();
            assert!(!key.is_empty());
        }
    }

    #[test]
    fn given_minimal_world_when_get_state_then_json_is_still_valid() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(1, 1); // Minimum population of 1
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        let json = world.get_state();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("dimensions").is_some());
        assert!(parsed.get("organisms").is_some());
        assert!(parsed.get("regions").is_some());

        let organisms = parsed.get("organisms").unwrap().as_array().unwrap();
        assert_eq!(organisms.len(), 1);
    }

    #[test]
    fn given_world_with_dead_organisms_when_get_state_then_dead_flag_is_serialized() {
        let bounds: Vec<RangeInclusive<f64>> = vec![0.0..=1.0];
        let gc = GlobalConstants::new(5, 2);
        let world_fn: Box<dyn WorldFunction> = Box::new(DummyFn);
        let world = super::super::World::new(&bounds, gc, world_fn);

        // Mark first organism as dead
        world.organisms.iter().next().unwrap().mark_dead();

        let json = world.get_state();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let organisms = parsed.get("organisms").unwrap().as_array().unwrap();
        let dead_count = organisms
            .iter()
            .filter(|o| o.get("is_dead").unwrap().as_bool().unwrap())
            .count();

        assert_eq!(dead_count, 1);
    }
}
