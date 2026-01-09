//! World module containing fitness function traits and world orchestration.

pub mod dimensions;
pub mod new;
pub mod regions;
pub mod single_valued_function;
pub mod world_function;
pub mod world_struct;

// Training run steps
pub mod age_and_cull;
pub mod calculate_region_keys;
pub mod get_best_organism;
pub mod get_best_params;
pub mod get_best_score;
pub mod get_state_for_web;
pub mod process_epoch;
pub mod reproduction;
pub mod setup_world;
pub mod training_run;

pub use dimensions::Dimensions;
pub use regions::Regions;
pub use single_valued_function::SingleValuedFunction;
pub use world_function::WorldFunction;
pub use world_struct::World;
