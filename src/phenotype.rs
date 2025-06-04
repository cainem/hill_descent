use crate::gamete::Gamete;
use rand::Rng;

/// A Phenotype is constructed from a pair of gametes.
#[derive(Debug, Clone, PartialEq)]
pub struct Phenotype {
    /// The first gamete.
    gamete1: Gamete,
    /// The second gamete.
    gamete2: Gamete,
    /// The expressed parameter values derived from the two gametes.
    expressed: Vec<f64>,
}

impl Phenotype {
    /// Creates a new Phenotype from two gametes, computing expressed values using the given RNG.
    pub fn new<R: Rng>(gamete1: Gamete, gamete2: Gamete, rng: &mut R) -> Self {
        let expressed = Self::compute_expressed(&gamete1, &gamete2, rng);
        Self {
            gamete1,
            gamete2,
            expressed,
        }
    }

    /// Returns references to the two gametes.
    pub fn gametes(&self) -> (&Gamete, &Gamete) {
        (&self.gamete1, &self.gamete2)
    }

    /// Returns the expressed parameter values.
    pub fn expressed_values(&self) -> &[f64] {
        &self.expressed
    }

    /// Compute expressed values per PDD regression rules.
    fn compute_expressed<R: Rng>(g1: &Gamete, g2: &Gamete, rng: &mut R) -> Vec<f64> {
        let loci1 = g1.loci();
        let loci2 = g2.loci();
        assert_eq!(
            loci1.len(),
            loci2.len(),
            "Gametes must have same number of loci"
        );
        let max_u64 = u64::MAX as f64;
        let mut result = Vec::with_capacity(loci1.len());
        for (l1, l2) in loci1.iter().zip(loci2.iter()) {
            let c1 = l1.adjustment().checksum();
            let c2 = l2.adjustment().checksum();
            let (a, b) = if c1 <= c2 { (l1, l2) } else { (l2, l1) };
            let ca = a.adjustment().checksum() as f64 / max_u64;
            let cb = b.adjustment().checksum() as f64 / max_u64;
            let midpoint = (ca + cb) / 2.0;
            let r = rng.gen_range(0.0..1.0);
            let value = if a.adjustment().checksum() == b.adjustment().checksum() {
                if r < 0.5 {
                    a.value().get()
                } else {
                    b.value().get()
                }
            } else if r <= midpoint {
                a.value().get()
            } else {
                b.value().get()
            };
            result.push(value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locus::Locus;
    use crate::locus_adjustment::{DirectionOfTravel, LocusAdjustment};
    use crate::parameter::Parameter;
    use rand::thread_rng;

    fn create_test_locus(val: f64) -> Locus {
        let param = Parameter::new(val);
        let adj = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        Locus::new(param, adj, false)
    }

    fn create_test_gamete(vals: &[f64]) -> Gamete {
        let loci = vals.iter().map(|&v| create_test_locus(v)).collect();
        Gamete::new(loci)
    }

    #[test]
    fn given_two_gametes_when_new_then_fields_are_set() {
        let g1 = create_test_gamete(&[1.0, 2.0]);
        let g2 = create_test_gamete(&[3.0, 4.0]);
        let mut rng = thread_rng();
        let ph = Phenotype::new(g1.clone(), g2.clone(), &mut rng);
        assert_eq!(ph.gametes(), (&g1, &g2));
    }

    #[test]
    fn compute_expressed_equal_checksums_choose_first() {
        use rand::rngs::mock::StepRng;
        let mut rng = StepRng::new(0, 0);
        let l1 = create_test_locus(1.0);
        let l2 = create_test_locus(2.0);
        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);
        let ph = Phenotype::new(g1, g2, &mut rng);
        assert_eq!(ph.expressed_values(), &[1.0]);
    }

    #[test]
    fn compute_expressed_equal_checksums_choose_second() {
        use rand::rngs::mock::StepRng;
        let mut rng = StepRng::new(u64::MAX, 0);
        let l1 = create_test_locus(1.0);
        let l2 = create_test_locus(2.0);
        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);
        let ph = Phenotype::new(g1, g2, &mut rng);
        assert_eq!(ph.expressed_values(), &[2.0]);
    }

    #[test]
    fn compute_expressed_unequal_checksums_small_r() {
        use rand::rngs::mock::StepRng;
        let mut rng = StepRng::new(0, 0);
        let adj1 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        let adj2 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Subtract, false);
        let l1 = Locus::new(Parameter::new(3.0), adj1.clone(), false);
        let l2 = Locus::new(Parameter::new(4.0), adj2.clone(), false);
        let c1 = adj1.checksum();
        let c2 = adj2.checksum();
        let expected = if c1 <= c2 { 3.0 } else { 4.0 };
        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);
        let ph = Phenotype::new(g1.clone(), g2.clone(), &mut rng);
        assert_eq!(ph.expressed_values(), &[expected]);
    }

    #[test]
    fn compute_expressed_unequal_checksums_large_r() {
        use rand::rngs::mock::StepRng;
        let mut rng = StepRng::new(u64::MAX, 0);
        let adj1 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Add, false);
        let adj2 = LocusAdjustment::new(Parameter::new(0.0), DirectionOfTravel::Subtract, false);
        let l1 = Locus::new(Parameter::new(3.0), adj1.clone(), false);
        let l2 = Locus::new(Parameter::new(4.0), adj2.clone(), false);
        let c1 = adj1.checksum();
        let c2 = adj2.checksum();
        let expected = if c1 > c2 { 3.0 } else { 4.0 };
        let g1 = Gamete::new(vec![l1.clone()]);
        let g2 = Gamete::new(vec![l2.clone()]);
        let ph = Phenotype::new(g1.clone(), g2.clone(), &mut rng);
        assert_eq!(ph.expressed_values(), &[expected]);
    }

    #[test]
    #[should_panic(expected = "Gametes must have same number of loci")]
    fn compute_expressed_mismatched_lengths_panics() {
        let mut rng = thread_rng();
        // g1 has one locus, g2 has two
        let l = create_test_locus(1.0);
        let g1 = Gamete::new(vec![l.clone()]);
        let g2 = Gamete::new(vec![l.clone(), create_test_locus(2.0)]);
        let _ = Phenotype::new(g1, g2, &mut rng);
    }

    #[test]
    fn compute_expressed_multi_loci_length() {
        use rand::rngs::mock::StepRng;
        let mut rng = StepRng::new(0, 1);
        let vals = [1.0, 2.0, 3.0, 4.0];
        let g1 = create_test_gamete(&vals);
        let g2 = create_test_gamete(&vals);
        let ph = Phenotype::new(g1, g2, &mut rng);
        assert_eq!(ph.expressed_values().len(), vals.len());
    }
}
