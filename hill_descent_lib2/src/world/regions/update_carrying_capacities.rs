//! Carrying capacity calculation for regions.

use super::Regions;

impl Regions {
    /// Updates carrying capacities for all regions based on relative fitness.
    ///
    /// Uses the inverse fitness formula from the PDD:
    /// capacity_i = P * (1/score_i) / sum(1/score_j for all j)
    ///
    /// Where P is the total population size.
    pub fn update_carrying_capacities(&mut self) {
        todo!("Implement update_carrying_capacities")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_regions_with_scores_when_update_capacities_then_capacities_calculated() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_region_with_lower_score_when_update_capacities_then_higher_capacity() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_all_capacities_when_summed_then_equals_population_size() {
        todo!()
    }
}
