use rand::Rng;

use crate::world::{organisms::Organism, regions::region::Region};

impl Region {
    pub fn repopulate<R: Rng>(&mut self, rng: &mut R, region_key: &[usize]) -> Vec<Organism> {

        // calculate the difference between the current number of organisms and the carrying capacity

        // call reproduce and add new organisms

        // add the organisms to the region (that are still are in this region)

        // return the remainder

        todo!()
    }
}
