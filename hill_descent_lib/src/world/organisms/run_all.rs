use super::Organisms;
use crate::world::world_function::WorldFunction;

impl Organisms {
    /// Runs the supplied `WorldFunction` for every organism in the collection and updates
    /// their scores.
    ///
    /// This is a thin wrapper over `Organism::run` and exists purely for convenience so that
    /// higher-level code such as `World::training_run` can express the whole fitness-evaluation
    /// step with a single call.
    #[cfg_attr(
        feature = "enable-tracing",
        tracing::instrument(level = "debug", skip(self, function, inputs, known_outputs))
    )]
    pub fn run_all(
        &self,
        function: &dyn WorldFunction,
        inputs: &[f64],
        known_outputs: Option<&[f64]>,
    ) {
        for organism in self.organisms.iter() {
            organism.run(function, inputs, known_outputs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Organisms;
    use crate::phenotype::Phenotype;
    use crate::world::organisms::organism::Organism;
    use crate::world::world_function::WorldFunction;
    use std::sync::Arc;

    // Mock WorldFunction that returns a constant vector [1.0, 1.0] to simplify scoring.
    #[derive(Debug)]
    struct DummyFn;
    impl WorldFunction for DummyFn {
        fn run(&self, _p: &[f64], _v: &[f64]) -> Vec<f64> {
            vec![1.0, 1.0] // constant output to make scoring deterministic
        }
    }

    fn make_organisms(count: usize) -> Organisms {
        let expressed = vec![0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0];
        let phenotype = Arc::new(Phenotype::new_for_test(expressed));
        let organisms: Vec<Organism> = (0..count)
            .map(|_| Organism::new(Arc::clone(&phenotype), 0, (None, None)))
            .collect();
        Organisms::new_from_organisms(organisms)
    }

    #[test]
    fn given_multiple_organisms_when_run_all_then_every_score_is_set() {
        let orgs = make_organisms(3);
        let inputs = vec![0.0, 0.0];
        let known_outputs = vec![1.0, 1.0];
        let wf = DummyFn;

        // Before run_all none of the organisms should have a score.
        assert!(orgs.iter().all(|o| o.score().is_none()));

        orgs.run_all(&wf, &inputs, Some(&known_outputs));

        // After run_all every organism should have Some(score)
        assert!(orgs.iter().all(|o| o.score().is_some()));
    }
}
