//! Combined epoch processing for all organisms.

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::World;
use crate::NUM_SYSTEM_PARAMETERS;
use crate::organism::ProcessEpochResult;
use crate::world::dimensions::Dimensions;
use crate::world::regions::{OrganismEntry, RegionKey};

impl World {
    /// Processes an epoch for all organisms using parallel iterators.
    ///
    /// Returns (dimensions_changed, dead_organism_ids, dead_per_region).
    /// The `dead_per_region` map is used for gap-filling reproduction.
    pub fn process_epoch_all(
        &mut self,
        training_data_index: usize,
    ) -> (bool, Vec<u64>, HashMap<RegionKey, usize>) {
        self.process_epoch_all_with_subdivided_dims(training_data_index, &[])
    }

    /// Processes an epoch for all organisms, with knowledge of which dimensions were subdivided.
    ///
    /// This variant is called after dimension subdivision (not OOB expansion) to ensure
    /// organisms recalculate their region keys for the subdivided dimensions.
    ///
    /// # Arguments
    ///
    /// * `training_data_index` - Index into shared training data
    /// * `subdivided_dims` - Indices of dimensions that were subdivided (need region key recalc)
    ///
    /// Returns (dimensions_changed, dead_organism_ids, dead_per_region).
    pub fn process_epoch_all_with_subdivided_dims(
        &mut self,
        training_data_index: usize,
        subdivided_dims: &[usize],
    ) -> (bool, Vec<u64>, HashMap<RegionKey, usize>) {
        let mut dimensions_changed = false;

        // Increment dimension version at start of each epoch to invalidate cached results
        // This ensures organisms are re-evaluated each epoch (age increment, etc.)
        // Further increments happen on OOB to track retry iterations
        self.dimension_version += 1;

        // If dimensions were subdivided, we need to send the updated dimensions to all organisms
        // so they can recalculate their region keys with the new interval counts.
        let mut dimensions_to_send: Option<Arc<Dimensions>> = if !subdivided_dims.is_empty() {
            Some(Arc::clone(&self.dimensions))
        } else {
            None
        };
        // Start with subdivided dimensions (for dimension subdivision), then add OOB dimensions
        let mut changed_since_last_attempt: Vec<usize> = subdivided_dims.to_vec();

        let (dead_organism_ids, dead_per_region) = loop {
            let dim_version = self.dimension_version;

            // Capture loop variables by reference to avoid cloning on each iteration
            let current_dims_arg = &dimensions_to_send;
            let current_changed_dims = &changed_since_last_attempt;

            struct UpdateResult {
                id: u64,
                is_oob: bool,
                oob_dims: Vec<usize>,
                is_dead: bool,
                entry: Option<OrganismEntry>,
                region_key: Option<RegionKey>,
            }

            let results: Vec<UpdateResult> = self
                .organisms
                .par_iter()
                .map(|(_, org)| {
                    // First check if we can use cached result (score caching optimization)
                    if org.is_epoch_complete(dim_version) {
                        // Already processed successfully - return cached result
                        if let Some(ProcessEpochResult::Ok {
                            should_remove,
                            region_key,
                            score,
                            new_age,
                        }) = org.get_cached_epoch_result()
                        {
                            return UpdateResult {
                                id: org.id(),
                                is_oob: false,
                                oob_dims: Vec::new(),
                                is_dead: should_remove,
                                entry: Some(OrganismEntry::new(org.id(), new_age, Some(score))),
                                region_key: Some(region_key),
                            };
                        }
                    }

                    // Need to process - uses interior mutability (atomics/mutex)
                    let res = org.process_epoch(
                        current_dims_arg.clone(),
                        dim_version,
                        current_changed_dims.clone(),
                        training_data_index,
                    );

                    match res {
                        ProcessEpochResult::Ok {
                            should_remove,
                            region_key,
                            score,
                            new_age,
                        } => UpdateResult {
                            id: org.id(),
                            is_oob: false,
                            oob_dims: Vec::new(),
                            is_dead: should_remove,
                            entry: Some(OrganismEntry::new(org.id(), new_age, Some(score))),
                            region_key: Some(region_key),
                        },
                        ProcessEpochResult::OutOfBounds {
                            dimensions_exceeded,
                        } => UpdateResult {
                            id: org.id(),
                            is_oob: true,
                            oob_dims: dimensions_exceeded,
                            is_dead: false,
                            entry: None,
                            region_key: None,
                        },
                    }
                })
                .collect();

            // Check for OOB
            let mut oob_dims: Vec<usize> = Vec::new();
            let mut has_oob = false;

            for res in &results {
                if res.is_oob {
                    has_oob = true;
                    for &d in &res.oob_dims {
                        if !oob_dims.contains(&d) {
                            oob_dims.push(d);
                        }
                    }
                }
            }

            if !has_oob {
                // Success!

                // Update regions
                let mut region_entries = Vec::new();
                let mut best_in_batch_score = f64::MAX;
                let mut best_in_batch_id = None;

                for res in &results {
                    if let (Some(entry), Some(key)) = (&res.entry, &res.region_key) {
                        region_entries.push((key.clone(), entry.clone()));

                        if let Some(s) = entry.score().filter(|&s| s < best_in_batch_score) {
                            best_in_batch_score = s;
                            best_in_batch_id = Some(entry.id());
                        }
                    }
                }
                self.regions.populate(region_entries);

                // Update global best
                if best_in_batch_id.is_some() && best_in_batch_score < self.best_score {
                    self.best_score = best_in_batch_score;
                    self.best_organism_id = best_in_batch_id;

                    // Fetch params for best organism using direct O(1) HashMap lookup
                    if let Some(best_id) = best_in_batch_id
                        && let Some(org) = self.organisms.get(&best_id)
                    {
                        let expressed = org.phenotype().expressed_values();
                        if expressed.len() > NUM_SYSTEM_PARAMETERS {
                            self.best_params = expressed[NUM_SYSTEM_PARAMETERS..].to_vec();
                        }
                    }
                }

                // Collect dead IDs and count per region
                let mut dead_ids = Vec::new();
                let mut dead_per_region_map: HashMap<RegionKey, usize> = HashMap::new();

                for res in &results {
                    if res.is_dead {
                        dead_ids.push(res.id);
                        if let Some(key) = &res.region_key {
                            *dead_per_region_map.entry(key.clone()).or_insert(0) += 1;
                        }
                    }
                }

                break (dead_ids, dead_per_region_map);
            }

            // Handle Out Of Bounds
            dimensions_changed = true;
            let mut new_dimensions = (*self.dimensions).clone();
            new_dimensions.expand_bounds_multiple(&oob_dims);
            self.dimensions = Arc::new(new_dimensions);
            self.dimension_version += 1;

            // Prepare for retry
            changed_since_last_attempt = oob_dims;
            dimensions_to_send = Some(Arc::clone(&self.dimensions));
        };

        (dimensions_changed, dead_organism_ids, dead_per_region)
    }
}
