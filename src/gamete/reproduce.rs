use crate::parameters::system_parameters::SystemParameters;

use super::Gamete;
use rand::Rng;
use rand::seq::SliceRandom;

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
        let mut points: Vec<usize> = (1..len).collect();
        points.shuffle(rng);
        points.truncate(crossovers);
        points.sort_unstable();
        // Perform crossover
        let mut offspring1 = Vec::with_capacity(len);
        let mut offspring2 = Vec::with_capacity(len);
        let mut use_p1 = true;
        let mut cps = points.into_iter();
        let mut next_cp = cps.next();
        for i in 0..len {
            if let Some(cp) = next_cp {
                if cp == i {
                    use_p1 = !use_p1;
                    next_cp = cps.next();
                }
            }
            if use_p1 {
                offspring1.push(parent1.loci()[i].mutate(rng, sys));
                offspring2.push(parent2.loci()[i].mutate(rng, sys));
            } else {
                offspring1.push(parent2.loci()[i].mutate(rng, sys));
                offspring2.push(parent1.loci()[i].mutate(rng, sys));
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
    use rand::rngs::mock::StepRng;

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
        let mut rng = StepRng::new(0, 0);
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
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::default();
        let _ = Gamete::reproduce(&g1, &g2, 0, &mut rng, &sys);
    }

    #[test]
    #[should_panic(expected = "Number of crossovers must satisfy len > 2 * crossovers")]
    fn reproduce_too_many_crossovers_panics() {
        let g1 = create_test_gamete(&[1.0, 2.0, 3.0]);
        let g2 = create_test_gamete(&[4.0, 5.0, 6.0]);
        let mut rng = StepRng::new(0, 0);
        let sys = SystemParameters::default();
        let _ = Gamete::reproduce(&g1, &g2, 2, &mut rng, &sys);
    }
}
