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

        // Increment epoch counter for reproduction RNG seeding
        self.epoch_count += 1;

        // Only increment dimension version when dimensions actually change.
        // This allows organisms to use cached region keys when dimensions are stable.
        // Further increments happen on OOB to track retry iterations.
        if !subdivided_dims.is_empty() {
            self.dimension_version += 1;
        }

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

            // Data structure to collect results from each thread
            struct SuccessData {
                region_entries: Vec<(RegionKey, OrganismEntry)>,
                dead_ids: Vec<u64>,
                dead_per_region_map: HashMap<RegionKey, usize>,
                best_score: f64,
                best_id: Option<u64>,
            }

            enum StepResult {
                Oob(Vec<usize>),
                Success(SuccessData),
            }

            let combined_result = self
                .organisms
                .par_iter()
                .fold(
                    || {
                        StepResult::Success(SuccessData {
                            region_entries: Vec::new(),
                            dead_ids: Vec::new(),
                            dead_per_region_map: HashMap::new(),
                            best_score: f64::MAX,
                            best_id: None,
                        })
                    },
                    |mut acc, (_, org)| {
                        // Check if we can use cached region key (dimensions unchanged)
                        // We still need to evaluate fitness (training data may differ) and increment age
                        let res = if org.has_valid_region_key(dim_version) {
                            org.process_epoch_with_cached_region_key(training_data_index)
                        } else {
                            org.process_epoch(
                                current_dims_arg.as_ref(),
                                dim_version,
                                current_changed_dims,
                                training_data_index,
                            )
                        };

                        match res {
                            ProcessEpochResult::Ok {
                                should_remove,
                                region_key,
                                score,
                                new_age,
                            } => {
                                if let StepResult::Success(ref mut data) = acc {
                                    let entry = OrganismEntry::new(org.id(), new_age, Some(score));
                                    data.region_entries.push((region_key.clone(), entry));

                                    if should_remove {
                                        data.dead_ids.push(org.id());
                                        *data.dead_per_region_map.entry(region_key).or_insert(0) +=
                                            1;
                                    }

                                    if score < data.best_score {
                                        data.best_score = score;
                                        data.best_id = Some(org.id());
                                    }
                                }
                            }
                            ProcessEpochResult::OutOfBounds {
                                dimensions_exceeded,
                            } => match acc {
                                StepResult::Oob(ref mut dims) => {
                                    for d in dimensions_exceeded {
                                        if !dims.contains(&d) {
                                            dims.push(d);
                                        }
                                    }
                                }
                                StepResult::Success(_) => {
                                    acc = StepResult::Oob(dimensions_exceeded);
                                }
                            },
                        }
                        acc
                    },
                )
                .reduce(
                    || {
                        StepResult::Success(SuccessData {
                            region_entries: Vec::new(),
                            dead_ids: Vec::new(),
                            dead_per_region_map: HashMap::new(),
                            best_score: f64::MAX,
                            best_id: None,
                        })
                    },
                    |acc1, acc2| match (acc1, acc2) {
                        (StepResult::Oob(mut d1), StepResult::Oob(d2)) => {
                            for d in d2 {
                                if !d1.contains(&d) {
                                    d1.push(d);
                                }
                            }
                            StepResult::Oob(d1)
                        }
                        (StepResult::Oob(d1), _) => StepResult::Oob(d1),
                        (_, StepResult::Oob(d2)) => StepResult::Oob(d2),
                        (StepResult::Success(mut s1), StepResult::Success(s2)) => {
                            s1.region_entries.extend(s2.region_entries);
                            s1.dead_ids.extend(s2.dead_ids);
                            for (k, v) in s2.dead_per_region_map {
                                *s1.dead_per_region_map.entry(k).or_insert(0) += v;
                            }
                            if s2.best_score < s1.best_score {
                                s1.best_score = s2.best_score;
                                s1.best_id = s2.best_id;
                            }
                            StepResult::Success(s1)
                        }
                    },
                );

            match combined_result {
                StepResult::Success(data) => {
                    // Success!
                    self.regions.populate(data.region_entries);

                    // Update global best
                    if data.best_id.is_some() && data.best_score < self.best_score {
                        self.best_score = data.best_score;
                        self.best_organism_id = data.best_id;

                        // Fetch params for best organism using direct O(1) HashMap lookup
                        if let Some(best_id) = data.best_id
                            && let Some(org) = self.organisms.get(&best_id)
                        {
                            let expressed = org.phenotype().expressed_values();
                            if expressed.len() > NUM_SYSTEM_PARAMETERS {
                                self.best_params = expressed[NUM_SYSTEM_PARAMETERS..].to_vec();
                            }
                        }
                    }

                    break (data.dead_ids, data.dead_per_region_map);
                }
                StepResult::Oob(oob_dims) => {
                    // Handle Out Of Bounds
                    dimensions_changed = true;
                    let mut new_dimensions = (*self.dimensions).clone();
                    new_dimensions.expand_bounds_multiple(&oob_dims);
                    self.dimensions = Arc::new(new_dimensions);
                    self.dimension_version += 1;

                    // Prepare for retry
                    changed_since_last_attempt = oob_dims;
                    dimensions_to_send = Some(Arc::clone(&self.dimensions));
                }
            }
        };

        (dimensions_changed, dead_organism_ids, dead_per_region)
    }
}
