//! Combined epoch processing for all organisms.

use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use super::World;
use crate::NUM_SYSTEM_PARAMETERS;
use crate::organism::ProcessEpochResult;
use crate::world::dimensions::Dimensions;
use crate::world::regions::{OrganismEntry, RegionKey};

impl World {
    /// Processes an epoch for all organisms using parallel iterators.
    ///
    /// Returns (dimensions_changed, dead_organism_ids).
    pub fn process_epoch_all(&mut self, training_data_index: usize) -> (bool, Vec<u64>) {
        let mut dimensions_changed = false;

        // Build index for O(1) organism lookups by ID
        let organism_index: FxHashMap<u64, usize> = self
            .organism_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        // These are only set after an out-of-bounds retry
        let mut dimensions_to_send: Option<Arc<Dimensions>> = None;
        let mut changed_since_last_attempt: Vec<usize> = Vec::new();

        let dead_organism_ids = loop {
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
                .map(|org_lock| {
                    let mut org = org_lock.write().unwrap();
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

                    // Fetch params for best organism using O(1) index lookup
                    if let Some(best_id) = best_in_batch_id
                        && let Some(&idx) = organism_index.get(&best_id)
                    {
                        let org = self.organisms[idx].read().unwrap();
                        let expressed = org.phenotype().expressed_values();
                        if expressed.len() > NUM_SYSTEM_PARAMETERS {
                            self.best_params = expressed[NUM_SYSTEM_PARAMETERS..].to_vec();
                        }
                    }
                }

                // Collect dead IDs
                break results
                    .iter()
                    .filter(|r| r.is_dead)
                    .map(|r| r.id)
                    .collect::<Vec<u64>>();
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

        (dimensions_changed, dead_organism_ids)
    }
}
