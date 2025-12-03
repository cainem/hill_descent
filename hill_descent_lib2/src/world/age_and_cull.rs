//! Age organisms and cull dead ones.

use super::World;

impl World {
    /// Increments age for all organisms and removes dead ones.
    ///
    /// Sends IncrementAgeRequest to all organisms and removes those
    /// that should_remove == true.
    pub fn age_and_cull(&mut self) {
        todo!("Implement age_and_cull")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Implementation pending"]
    fn given_young_organisms_when_age_and_cull_then_ages_increment() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_old_organisms_when_age_and_cull_then_removed() {
        todo!()
    }

    #[test]
    #[ignore = "Implementation pending"]
    fn given_mixed_ages_when_age_and_cull_then_only_old_removed() {
        todo!()
    }
}
