use crate::{NUM_SYSTEM_PARAMETERS, parameters::system_parameters::SystemParameters};

use super::Gamete;
use rand::Rng;

impl Gamete {
    /// Performs multi-point crossover with `crossovers` points and returns two offspring gametes.
    ///
    /// Panics if gametes differ in length or if `len <= 2 * crossovers`.
    pub fn reproduce<R: Rng>(
        parent1: &Gamete,
        parent2: &Gamete,
        crossovers: usize,
        rng: &mut R,
        sys: &SystemParameters,
    ) -> (Gamete, Gamete) {
        let len = parent1.len();
        assert_eq!(len, parent2.len(), "Gametes must have same number of loci");
        assert!(
            len > 2 * crossovers,
            "Number of crossovers must satisfy len > 2 * crossovers"
        );
        // Generate unique, sorted crossover points between 1 and len-1
        let mut points = Vec::with_capacity(crossovers);
        if crossovers > 0 {
            if crossovers == 1 {
                points.push(rng.random_range(1..len));
            } else {
                // For small number of crossovers, simple rejection sampling is faster than shuffling full range
                while points.len() < crossovers {
                    let cp = rng.random_range(1..len);
                    if !points.contains(&cp) {
                        points.push(cp);
                    }
                }
                points.sort_unstable();
            }
        }
        // Perform crossover using pooled buffers to avoid per-call heap allocation.
        // In the steady state, buffers are recycled from dropped gametes.
        let mut offspring1 = Self::take_buffer(len);
        let mut offspring2 = Self::take_buffer(len);
        let mut use_p1 = true;
        let mut cps = points.into_iter();
        let mut next_cp = cps.next();
        let dists = sys.mutation_distributions();
        for i in 0..len {
            if let Some(cp) = next_cp
                && cp == i
            {
                use_p1 = !use_p1;
                next_cp = cps.next();
            }
            if use_p1 {
                if i < NUM_SYSTEM_PARAMETERS {
                    // System parameters: use bounded mutation
                    offspring1.push(parent1.loci()[i].mutate(rng, &dists));
                    offspring2.push(parent2.loci()[i].mutate(rng, &dists));
                } else {
                    // Problem parameters: use unbounded mutation
                    offspring1.push(parent1.loci()[i].mutate_unbound(rng, &dists));
                    offspring2.push(parent2.loci()[i].mutate_unbound(rng, &dists));
                }
            } else if i < NUM_SYSTEM_PARAMETERS {
                // System parameters: use bounded mutation
                offspring1.push(parent2.loci()[i].mutate(rng, &dists));
                offspring2.push(parent1.loci()[i].mutate(rng, &dists));
            } else {
                // Problem parameters: use unbounded mutation
                offspring1.push(parent2.loci()[i].mutate_unbound(rng, &dists));
                offspring2.push(parent1.loci()[i].mutate_unbound(rng, &dists));
            }
        }
        (Gamete::new(offspring1), Gamete::new(offspring2))
    }
}

#[cfg(test)]
mod tests {
    use crate::gamete::Gamete;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameters::parameter::Parameter;
    use crate::parameters::system_parameters::SystemParameters;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    fn create_test_gamete(vals: &[f64]) -> Gamete {
        let loci = vals.iter().map(|val| create_test_locus(*val)).collect();
        Gamete::new(loci)
    }

    #[test]
    fn reproduce_zero_crossovers_returns_clones() {
        let g1 = create_test_gamete(&[1.0, 2.0, 3.0]);
        let g2 = create_test_gamete(&[4.0, 5.0, 6.0]);
        let mut rng = SmallRng::seed_from_u64(0);
        let sys = SystemParameters::default();
        let (o1, o2) = Gamete::reproduce(&g1, &g2, 0, &mut rng, &sys);
        assert_eq!(o1, g1);
        assert_eq!(o2, g2);
    }

    #[test]
    #[should_panic(expected = "Gametes must have same number of loci")]
    fn reproduce_mismatched_lengths_panics() {
        let g1 = create_test_gamete(&[1.0]);
        let g2 = create_test_gamete(&[1.0, 2.0]);
        let mut rng = SmallRng::seed_from_u64(0);
        let sys = SystemParameters::default();
        let _ = Gamete::reproduce(&g1, &g2, 0, &mut rng, &sys);
    }

    #[test]
    #[should_panic(expected = "Number of crossovers must satisfy len > 2 * crossovers")]
    fn reproduce_too_many_crossovers_panics() {
        let g1 = create_test_gamete(&[1.0, 2.0, 3.0]);
        let g2 = create_test_gamete(&[4.0, 5.0, 6.0]);
        let mut rng = SmallRng::seed_from_u64(0);
        let sys = SystemParameters::default();
        let _ = Gamete::reproduce(&g1, &g2, 2, &mut rng, &sys);
    }

    #[test]
    fn given_reproduce_when_system_parameters_exceed_bounds_then_values_are_clamped() {
        use crate::NUM_SYSTEM_PARAMETERS;

        // Create gametes with system parameters that will exceed bounds when mutated
        let mut loci1 = Vec::new();
        let mut loci2 = Vec::new();

        // Add system parameters (indices 0-6) with bounds that will be exceeded
        for _i in 0..NUM_SYSTEM_PARAMETERS {
            let param1 = Parameter::with_bounds(1.9, 1.0, 2.0);
            let param2 = Parameter::with_bounds(1.9, 1.0, 2.0);
            let adj = LocusAdjustment::new(
                Parameter::with_bounds(0.5, 0.0, 1.0),
                DirectionOfTravel::Add,
                false,
            );
            loci1.push(Locus::new(param1, adj.clone(), true)); // apply_flag = true
            loci2.push(Locus::new(param2, adj, true));
        }

        // Add one problem parameter (index 7+)
        let param1 = Parameter::with_bounds(1.9, 1.0, 2.0);
        let param2 = Parameter::with_bounds(1.9, 1.0, 2.0);
        let adj = LocusAdjustment::new(
            Parameter::with_bounds(0.5, 0.0, 1.0),
            DirectionOfTravel::Add,
            false,
        );
        loci1.push(Locus::new(param1, adj.clone(), true));
        loci2.push(Locus::new(param2, adj, true));

        let g1 = Gamete::new(loci1);
        let g2 = Gamete::new(loci2);

        let mut rng = SmallRng::seed_from_u64(0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // No mutations
        let (offspring1, _) = Gamete::reproduce(&g1, &g2, 0, &mut rng, &sys);

        // System parameters (0-6) should be clamped to 2.0
        for i in 0..NUM_SYSTEM_PARAMETERS {
            assert_eq!(
                offspring1.loci()[i].value().get(),
                2.0,
                "System parameter {i} should be clamped to bounds"
            );
        }

        // Problem parameter (7) should not be clamped and should be 2.4
        assert_eq!(
            offspring1.loci()[NUM_SYSTEM_PARAMETERS].value().get(),
            2.4,
            "Problem parameter should not be clamped"
        );
    }

    #[test]
    fn given_reproduce_when_problem_parameters_exceed_bounds_then_values_are_not_clamped() {
        use crate::NUM_SYSTEM_PARAMETERS;

        // Create gametes with only problem parameters (8 total, so indices 7+ are problem params)
        let mut loci1 = Vec::new();
        let mut loci2 = Vec::new();

        // Add system parameters (indices 0-6) - these will be clamped
        for _ in 0..NUM_SYSTEM_PARAMETERS {
            let param1 = Parameter::with_bounds(1.5, 1.0, 2.0);
            let param2 = Parameter::with_bounds(1.5, 1.0, 2.0);
            let adj = LocusAdjustment::new(
                Parameter::with_bounds(0.0, 0.0, 1.0),
                DirectionOfTravel::Add,
                false,
            );
            loci1.push(Locus::new(param1, adj.clone(), false)); // apply_flag = false
            loci2.push(Locus::new(param2, adj, false));
        }

        // Add problem parameter (index 7) that will exceed bounds
        let param1 = Parameter::with_bounds(1.9, 1.0, 2.0);
        let param2 = Parameter::with_bounds(1.9, 1.0, 2.0);
        let adj = LocusAdjustment::new(
            Parameter::with_bounds(0.5, 0.0, 1.0),
            DirectionOfTravel::Add,
            false,
        );
        loci1.push(Locus::new(param1, adj.clone(), true)); // apply_flag = true
        loci2.push(Locus::new(param2, adj, true));

        let g1 = Gamete::new(loci1);
        let g2 = Gamete::new(loci2);

        let mut rng = SmallRng::seed_from_u64(0);
        let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]); // No mutations
        let (offspring1, _) = Gamete::reproduce(&g1, &g2, 0, &mut rng, &sys);

        // System parameters should remain unchanged (apply_flag = false)
        for i in 0..NUM_SYSTEM_PARAMETERS {
            assert_eq!(
                offspring1.loci()[i].value().get(),
                1.5,
                "System parameter {i} should remain unchanged"
            );
        }

        // Problem parameter should exceed bounds: 1.9 + 0.5 = 2.4
        assert_eq!(
            offspring1.loci()[NUM_SYSTEM_PARAMETERS].value().get(),
            2.4,
            "Problem parameter should exceed original bounds without clamping"
        );
    }
}
