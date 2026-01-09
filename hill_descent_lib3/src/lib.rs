//! Hill Descent Library (v3 - Shared Memory / Rayon)

// Use mimalloc as the global allocator for better multi-threaded performance
//use mimalloc::MiMalloc;

//#[global_allocator]
//static GLOBAL: MiMalloc = MiMalloc;

pub mod format_score;
pub mod gamete;
pub mod gen_hybrid_range;
pub mod locus;
pub mod organism;
pub mod parameters;
pub mod phenotype;
pub mod tracing_init;
pub mod training_data;
pub mod world;

/// Minimum positive float (epsilon)
pub(crate) const E0: f64 = f64::MIN_POSITIVE;

/// Number of system parameters used by the genetic algorithm.
pub(crate) const NUM_SYSTEM_PARAMETERS: usize = 7;

// Re-export key types
pub use parameters::GlobalConstants;
pub use training_data::TrainingData;
pub use world::get_best_organism::BestOrganism;
pub use world::setup_world::setup_world;
pub use world::{SingleValuedFunction, World, WorldFunction};
