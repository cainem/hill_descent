use rand::Rng;

use crate::world::regions::{region::Region, Regions};

impl Regions {
    pub fn repopulate<R: Rng>(&mut self, rng: &mut R) {

        // for each region call repopulate

        // add the new organisms returned to their correct regions
    }
}