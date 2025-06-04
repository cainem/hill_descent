use crate::phenotype::Phenotype;

impl Phenotype {
    /// Calculates a valid number of crossover points for `Gamete::reproduce`.
    ///
    /// `Gamete::reproduce` requires `gamete_len > 2 * crossovers`.
    /// This function takes an `m3` value (which suggests a desired number of crossovers)
    /// and a `gamete_len`, then returns a number of crossovers that respects the constraint.
    ///
    /// # Arguments
    /// * `m3`: The system parameter value (typically from `SystemParameters::m3()`) suggesting crossovers.
    /// * `gamete_len`: The length of the gametes undergoing crossover.
    ///
    /// # Returns
    /// The calculated number of crossovers, adjusted to be valid. Returns 0 if `gamete_len` is 0,
    /// in which case `Gamete::reproduce` is expected to panic.
    pub fn calculate_crossovers(m3: f64, gamete_len: usize) -> usize {
        if gamete_len == 0 {
            // Gamete::reproduce will panic if gamete_len is 0, as 0 > 2*0 is false.
            return 0;
        }
        let desired_crossovers = m3.round().max(0.0) as usize;

        // Max crossovers such that `gamete_len > 2 * max_crossovers`.
        // This means `2 * max_crossovers <= gamete_len - 1`.
        // So, `max_crossovers <= (gamete_len - 1) / 2`.
        let max_allowed_crossovers = gamete_len.saturating_sub(1) / 2;
        desired_crossovers.min(max_allowed_crossovers)
    }
}

#[cfg(test)]
mod tests {
    use crate::phenotype::Phenotype; // To call Phenotype::calculate_crossovers

    #[test]
    fn test_calculate_crossovers() {
        assert_eq!(Phenotype::calculate_crossovers(1.0, 0), 0, "len 0");
        assert_eq!(Phenotype::calculate_crossovers(0.0, 5), 0, "len 5, m3=0.0");
        assert_eq!(Phenotype::calculate_crossovers(0.8, 5), 1, "len 5, m3=0.8 -> 1");
        assert_eq!(Phenotype::calculate_crossovers(2.0, 5), 2, "len 5, m3=2.0 -> 2"); // max_allowed = (5-1)/2 = 2
        assert_eq!(
            Phenotype::calculate_crossovers(3.0, 5),
            2,
            "len 5, m3=3.0 -> capped at 2"
        );
        assert_eq!(Phenotype::calculate_crossovers(1.0, 1), 0, "len 1, max_allowed=0");
        assert_eq!(Phenotype::calculate_crossovers(1.0, 2), 0, "len 2, max_allowed=0");
        assert_eq!(Phenotype::calculate_crossovers(1.0, 3), 1, "len 3, max_allowed=1");
        assert_eq!(Phenotype::calculate_crossovers(-1.0, 5), 0, "negative m3");
    }
}
