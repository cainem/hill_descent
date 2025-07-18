impl super::Regions {
    pub fn get_most_diverse_dimension(&self, _region_key: Vec<usize>) -> Option<usize> {
        // TODO
        // loop through the organisms looking for ones with the key provided.
        // for each matching organism track the number of distinct values the organism has for each dimension and track the standard deviation.

        // if the highest number of distinct values in any dimension is 1 then return None
        // else return the index of the dimensions with the most distinct values using highest standard deviation as a tie-breaker
        todo!();
    }
}
