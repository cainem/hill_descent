use crate::gamete::Gamete;
use rand::Rng;

/// Compute expressed values per PDD regression rules.
/// This function is intended for use within the phenotype module.
pub(super) fn compute_expressed<R: Rng>(g1: &Gamete, g2: &Gamete, rng: &mut R) -> Vec<f64> {
    let loci1 = g1.loci();
    let loci2 = g2.loci();
    if loci1.len() != loci2.len() {
        panic!("Gametes must have same number of loci");
    }
    let max_u64_f64 = u64::MAX as f64;
    let mut result = Vec::with_capacity(loci1.len());
    for (l1, l2) in loci1.iter().zip(loci2.iter()) {
        let c1 = l1.adjustment().checksum();
        let c2 = l2.adjustment().checksum();

        let value = if c1 == c2 {
            if rng.random_bool(0.5) {
                l1.value().get()
            } else {
                l2.value().get()
            }
        } else {
            let (a, b, ca, cb) = if c1 < c2 {
                (l1, l2, c1, c2)
            } else {
                (l2, l1, c2, c1)
            };
            let midpoint = (ca as f64 + cb as f64) / (2.0 * max_u64_f64);
            if rng.random_range(0.0..1.0) <= midpoint {
                a.value().get()
            } else {
                b.value().get()
            }
        };
        result.push(value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*; // For compute_expressed
    use crate::gamete::Gamete;
    use crate::locus::Locus;
    use crate::locus::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameters::parameter::Parameter;
    use crate::phenotype::tests::{create_test_gamete, create_test_locus};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    // Helper: find a seed for which the first r in [0,1) satisfies predicate
    fn find_seed_for_predicate<P: Fn(f64) -> bool>(predicate: P) -> u64 {
        for seed in 0u64..100_000 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let r = rng.random_range(0.0..1.0);
            if predicate(r) {
                return seed;
            }
        }
        panic!("No suitable seed found within search range");
    }

    #[test]
    fn given_equal_checksums_rng_chooses_first_when_compute_expressed_then_returns_first_value() {
        let seed = find_seed_for_predicate(|r| r < 0.5);
        let mut rng = SmallRng::seed_from_u64(seed);
        let l1 = create_test_locus(1.0);
        let l2 = create_test_locus(2.0);
        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);
        let expressed_values = compute_expressed(&g1, &g2, &mut rng);
        assert_eq!(expressed_values, &[1.0]);
    }

    #[test]
    fn given_equal_checksums_rng_chooses_second_when_compute_expressed_then_returns_second_value() {
        // Use a seed that makes first r close to 1.0 so second value is chosen
        let seed = find_seed_for_predicate(|r| r >= 0.999);
        let mut rng = SmallRng::seed_from_u64(seed);
        let l1 = create_test_locus(1.0);
        let l2 = create_test_locus(2.0);
        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);
        let expressed_values = compute_expressed(&g1, &g2, &mut rng);
        assert_eq!(expressed_values, &[2.0]);
    }

    #[test]
    fn given_unequal_checksums_rng_favors_smaller_checksum_locus_when_compute_expressed_then_returns_its_value()
     {
        let adj1 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false); // Smaller checksum
        let adj2 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Subtract, false); // Larger checksum
        let l1 = Locus::new(Parameter::new(3.0), adj1.clone(), false);
        let l2 = Locus::new(Parameter::new(4.0), adj2.clone(), false);

        assert!(adj1.checksum() < adj2.checksum());

        // compute midpoint as in compute_expressed
        let max_u64 = u64::MAX as f64;
        let ca = adj1.checksum() as f64 / max_u64;
        let cb = adj2.checksum() as f64 / max_u64;
        let midpoint = (ca + cb) / 2.0;

        // choose seed so r <= midpoint to select the smaller checksum locus (l1)
        let seed = find_seed_for_predicate(|r| r <= midpoint);
        let mut rng = SmallRng::seed_from_u64(seed);

        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);

        let expressed_values = compute_expressed(&g1, &g2, &mut rng);
        assert_eq!(expressed_values, &[3.0]);
    }

    #[test]
    fn given_unequal_checksums_rng_favors_larger_checksum_locus_when_compute_expressed_then_returns_its_value()
     {
        let adj1 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false); // Smaller checksum
        let adj2 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Subtract, false); // Larger checksum
        let l1 = Locus::new(Parameter::new(3.0), adj1.clone(), false);
        let l2 = Locus::new(Parameter::new(4.0), adj2.clone(), false);

        assert!(adj1.checksum() < adj2.checksum());

        // compute midpoint as in compute_expressed
        let max_u64 = u64::MAX as f64;
        let ca = adj1.checksum() as f64 / max_u64;
        let cb = adj2.checksum() as f64 / max_u64;
        let midpoint = (ca + cb) / 2.0;

        // choose seed so r > midpoint to select the larger checksum locus (l2)
        let seed = find_seed_for_predicate(|r| r > midpoint);
        let mut rng = SmallRng::seed_from_u64(seed);

        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);

        let expressed_values = compute_expressed(&g1, &g2, &mut rng);
        assert_eq!(expressed_values, &[4.0]);
    }

    #[test]
    #[should_panic(expected = "Gametes must have same number of loci")]
    fn given_mismatched_gamete_lengths_when_compute_expressed_then_panics() {
        let mut rng = SmallRng::seed_from_u64(0);
        let l = create_test_locus(1.0);
        let g1 = Gamete::new(vec![l.clone()]);
        let g2 = Gamete::new(vec![l.clone(), create_test_locus(2.0)]);
        let _ = compute_expressed(&g1, &g2, &mut rng);
    }

    #[test]
    fn given_multi_loci_gametes_when_compute_expressed_then_returns_correct_length_vector() {
        let mut rng = SmallRng::seed_from_u64(0);
        let vals1 = [1.0, 2.0, 3.0, 4.0];
        let vals2 = [5.0, 6.0, 7.0, 8.0];
        let g1 = create_test_gamete(&vals1);
        let g2 = create_test_gamete(&vals2);
        let expressed_values = compute_expressed(&g1, &g2, &mut rng);
        assert_eq!(expressed_values.len(), vals1.len());
    }
}
