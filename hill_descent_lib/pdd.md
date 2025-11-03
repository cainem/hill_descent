# Product Definition Document: N-Dimensional Hill Descent Algorithm

## 1. Introduction and Goal

**1.1. System Goal:** To define a system that optimizes a set of `n` hidden dimensions ($x_1, x_2, ..., x_n$) to minimize a given fitness function. This process is analogous to "descending" a fitness landscape.

**1.2. Primary Application Context:** The primary context of interest is the optimization of neural network parameters (weights and biases), but the system is designed for general applicability to problems expressible in a similar optimization framework.

## 2. Core Problem Definition

**2.1. System Function to Optimize:** The system parameters $x_1, x_2, ..., x_n$ are inputs to a generalized function, exemplified by a neural network:  
$NeuralNetwork(inputs, x_1, x_2, ..., x_n) = (R_1, R_2, ..., R_p)$  

Where:  
* `inputs`: Known, fixed inputs for a given problem instance.  
* $x_1, x_2, ..., x_n$: The `n` variable parameters (dimensions) to be optimized by the algorithm. These can include weights, biases, and other tunable system parameters like mutation probabilities.  
* $(R_1, R_2, ..., R_p)$: The `p` output values produced by the function using the current set of parameters $x_i$.

**2.2. Fitness Function:** The fitness function measures the "goodness" of a set of parameters $(x_1, ..., x_n)$ by comparing the produced outputs $(R_1, ..., R_p)$ to a set of known target outputs $(A_1, ..., A_p)$ for the given `inputs`. The goal is to minimize this fitness value.  

The fitness function is defined as the Euclidean distance between the outputs and target outputs:  
$fitness(inputs, x_1, x_2, ..., x_n) = \sqrt{\sum_{i=1}^{p} (A_i - R_i)^2}$  

Where:  
* $(A_1, ..., A_p)$: The known target outputs for the given `inputs`.  
* $(R_1, ..., R_p)$: The outputs produced by the `NeuralNetwork` (or equivalent function) with the current parameters.  
* Zero fitness scores represent perfect matches and are handled as infinite fitness in carrying capacity calculations.

## 3. System Overview

**3.1. Approach:** The system employs a genetic algorithm (GA) to iteratively search for an optimal set of parameters $(x_1, ..., x_n)$.

**3.2. Population:** A population of `P` individual "organisms" is maintained. Each organism represents a potential solution (a specific set of values for $x_1, ..., x_n$).

**3.3. Competition and Selection:** Organisms compete based on their fitness scores. Organisms that perform relatively well (lower fitness scores) are given opportunities to reproduce.

**3.4. Evolution:** "Movement" in the n-dimensional solution space occurs through reproduction. Offspring inherit genetic material (DNA) from their parent(s), with variations introduced through mechanisms like crossover and mutation, leading to new points in the solution space.

**3.5. Environment:** Organisms exist in a simulated n-dimensional "space" and operate within a round-based system.

## 4. Key Components

### 4.1. Organism

An organism is defined by its DNA, which determines its position in the n-dimensional solution space and its characteristics.

**4.1.1. DNA:** 
* Composed of `n` locus pairs, where `n` is the number of dimensions being optimized.  
* Each locus pair corresponds to one dimension $x_j$.  
* DNA is fixed at birth for an organism.  
* DNA is conceptualized as two separate "gametes," one from each parent (in sexual reproduction), aiding the reproduction process.

**4.1.2. Locus Pair:** 
* Represents a single dimension $x_j$.  
* Consists of two individual loci, one inherited from each parent.  
* The "expressed" value for the dimension $x_j$ is determined from one of the two loci in the pair based on a recessiveness mechanism (see Section 4.1.3.c.iv and Section 5.2.3.1).

**4.1.3. Locus:** Each locus has the following properties:

* **a) Value (LocusValue):** 
    * A floating-point number representing the locus's contribution to a dimension's value.  
    * Can be positive or negative.  
    * Two modes:  
        * Unconstrained: Allowed range is $\pm M$, where M is the largest representable floating-point number. NaN is not allowed.  
        * Constrained: Values are bounded, e.g., between 0.0 and 1.0 (typical for neural network weights or probabilities). The specific bounds depend on the parameter type.

* **b) Adjustment (LocusAdjustment):** 
    * A structure defining a potential modification to `LocusValue`.  
    * Properties of `LocusAdjustment`:  
        * **i. AdjustmentValue:** * A positive floating-point number.  
            * Two modes: Unconstrained, or constrained (e.g., bounded to 1.0).  
            * This `AdjustmentValue` itself mutates by doubling or halving (see Section 5.2.4.3).  
        * **ii. DirectionOfTravel:** * A flag indicating whether `AdjustmentValue` is to be added (+) or subtracted (-) from `LocusValue`.  
            * Mutates with probability $m_4$ (see Section 5.2.4.2).  
        * **iii. DoublingOrHalvingFlag:** * A boolean flag determining if `AdjustmentValue` doubles or halves upon its own mutation event.  
            * Mutates with probability $m_3$ (see Section 5.2.4.1).  
            * Also flips if `DirectionOfTravel` mutates.  
        * **iv. Checksum:** * A positive 64-bit integer hash value calculated using the XXH3 algorithm from the entire state of the `LocusAdjustment` (i.e., its `AdjustmentValue`, `DirectionOfTravel`, and `DoublingOrHalvingFlag`).  
            * Used to determine recessiveness for locus expression (see Section 5.2.3.1).

* **c) ApplyAdjustmentFlag:** 
    * A boolean flag.  
    * If `true`, the `LocusAdjustment` (specifically, its `AdjustmentValue` modified by `DirectionOfTravel`) is applied to the `LocusValue` of the offspring's locus during reproduction.  
    * Mutation probabilities for this flag (see Section 5.2.4.4):  
        * False to True: $m_1$ (e.g., 10%).  
        * True to False: $m_2$ (e.g., 50%).

**4.1.4. Age:** 
* Each organism tracks its age in rounds.  
* Initialized with a random age at startup to avoid mass die-offs.  
* Maximum age ($A_{max}$) can be one of the `n` evolvable parameters (and must be a bounded parameter). Organisms reaching $A_{max}$ are removed.

### 4.2. Space

**4.2.1. N-Dimensionality (Parameter Space vs. Niching Space):** 
* The total parameter space an organism possesses is `n`-dimensional, corresponding to the `n` parameters ($x_1, ..., x_n$) being optimized. This `n` includes both problem-specific parameters (e.g., neural network weights and biases) and evolvable system parameters (e.g., mutation rates, maximum age).
* However, for the purpose of spatial niching and calculating an organism's `dimensions_key`, only the subset of these `n` parameters that are **problem-specific** are considered. Let $n_p$ be the number of these problem-specific parameters.
* An organism's position in the *niching space* (which is $n_p$-dimensional and used for region assignment) is determined by the expressed values of these $n_p$ problem-specific parameters. The `dimensions_key` reflects this position.

**4.2.2. Regions (n-orthotopes):** 
* The *niching space* (which is $n_p$-dimensional, as defined by the problem-specific parameters) is divided into up to `Z` regions. `Z` is a configurable constant, and a typical guideline might be $Z \ge n_p^2$.
* Regions are $n_p$-dimensional hyperrectangles (n-orthotopes).  
* Division process:  
    * Initially, a bounding box encompassing the ranges of the $n_p$ problem-specific parameters across the population is determined (see Section 4.2.3).  
    *   **Division Trigger:** Division occurs when the number of populated regions is less than the target number of regions, `Z`, and there is an opportunity to increase spatial resolution.
    *   **Division Strategy:** Instead of a simple round-robin division, the system uses an adaptive strategy based on population distribution:
        1.  **Identify Most Populous Region:** The region containing the highest number of organisms is selected as the candidate for division. This focuses the division effort on the most densely populated area of the solution space.
        2.  **Select Dimension to Divide:** Within this most populous region, the system determines the "most diverse" problem-specific dimension to split. Diversity is measured using a two-tiered approach:
            *   **Primary Criterion: Uniqueness.** The dimension with the highest number of distinct phenotype values among the organisms in the region is considered the most diverse.
            *   **Tie-Breaker: Standard Deviation.** If multiple dimensions have the same number of unique values, the one with the largest standard deviation is chosen. This ensures that the split happens along the dimension with the widest spread of organisms.
        3.  **Execution:** The selected dimension is then divided. This is a global operation that applies across the entire space. By incrementing the number of divisions in that single dimension, the total number of potential regions in the n-dimensional space is doubled.
    *   **Termination:** This division process continues until either the target number of regions `Z` is reached, or no further meaningful divisions can be made (e.g., the most populous region has no diversity in any of its dimensions, meaning all organisms within it are at the same point).
    *   Unoccupied regions resulting from division are of no interest.

**4.2.3. Bounding Box:** 
* An n-dimensional hyperrectangles that encompasses all organisms in the population.  
* Initial Bounding Box: Calculated at startup to be twice the size (in each dimension) required to hold all initial organisms.  
* Recalculation: If any new organism (offspring) falls outside the current bounding box, a new bounding box is calculated (again, twice the required size in each dimension), and all regions are recalculated.

**4.2.4. Carrying Capacity ($P_i$):** 
* The number of organisms a region `i` can support.  
* Calculated based on the relative fitness performance of regions using an inverse fitness formula.

**Carrying Capacity Formula:**
* Each region's capacity is allocated proportionally to its inverse fitness: $P_i = P \cdot \frac{1/F_i}{\sum_{j=1}^{R} (1/F_j)}$
    * $P$: Total population capacity
    * $F_i$: Minimum fitness in region `i` (fitness can be zero, negative, or positive)
    * $R$: Total number of populated regions
    * This formula rewards regions with better performance (lower fitness scores) with higher carrying capacity
    * **Special handling**: Regions with zero fitness (perfect scores) get infinite inverse fitness and receive priority allocation of all available capacity

**Recalculation Triggers:**
* Required if previously empty regions become populated, previously populated regions become empty, or if the overall bounding box changes.
* Recalculation occurs when region structure changes (dimension splits, boundary adjustments).

## 5. Key Processes

### 5.1. Initialization (Startup)

* **5.1.1. Determine `n`:** Calculate the total number of dimensions (weights, biases, and other evolvable system parameters like mutation probabilities $m_1..m_5$ and $A_{max}$).

* **5.1.2. Parameter Types:** For each dimension, know if it's unconstrained (float limits) or constrained (e.g., 0.0 to 1.0 for probabilities).

* **5.1.3. Create Initial Population:** * Generate `P` organisms. `P` is a hardcoded configuration constant.  
    * For each organism, seed its `n` locus pairs. For each locus:  
        * `LocusValue`: Random value within its defined bounds (constrained or unconstrained).  
        * `LocusAdjustment` (properties like `AdjustmentValue`, `DirectionOfTravel`, `DoublingOrHalvingFlag`): Random values within their respective bounds/options.  
        * `Checksum`: Calculated using XXH3 from the adjustment state.  
        * `ApplyAdjustmentFlag`: Randomly chosen (true or false).  
    * Assign each organism a random initial age (up to $A_{max}$) to stagger deaths.

* **5.1.4. Initialize Space:** 
    * Calculate the initial bounding box.  
    * Perform initial region division.  
    * Assign each organism to its respective region.  
    * Calculate initial carrying capacities for all populated regions.

### 5.2. Round-Based Operation

The system proceeds in discrete rounds. Each round involves:

**5.2.1. Problem Presentation & Fitness Evaluation:** 
* Present the same known problem (a specific set of `inputs` and corresponding target outputs $A_1, ..., A_p$) to all organisms in the population.  
* For each organism, calculate its output $(R_1, ..., R_p)$ using its expressed locus pair values as parameters $x_1, ..., x_n$.  
* Calculate the fitness score for each organism using the fitness function (Section 2.2).

**5.2.2. Ranking:** 
* Perform a separate ranking of organisms within each populated region based on their fitness scores (lowest score is best).  
* Tie-breaking rules for ranking:
    1.  Fitness score (lower is better).  
    2.  Age (older is ranked higher).  
    • If both fitness and age are identical, ordering is considered **arbitrary**; no further rule is defined.

**5.2.3. Reproduction (Occurs per Region):** 
* For each region, determine the required number of offspring, `r`, to fill the gap. This is based on the difference between the number of organisms that will be left after removing those at $A_{max}$ and the region's carrying capacity $P_i$. Organisms at $A_{max}$ don't count towards current population for capacity but can still reproduce.  

**Selection for Reproduction:** 
* Select the top `r` organisms from the region's ranked list.  
* If the number of organisms in the region (eligible for count) is less than `r`, all organisms in that region are selected.  

**Pairing and Reproduction Type (Extreme Pairing Strategy):** 
All reproduction is sexual. Organisms are paired using an "extreme pairing" strategy that pairs the best performers with the worst performers:
* If `r` is even: Organisms are paired as follows: 1st with rth, 2nd with (r-1)th, 3rd with (r-2)th, etc. Each pair produces two offspring.
* If `r` is odd: The top-ranked organism is duplicated in the pairing list. The resulting even-sized list (r+1 organisms) is then paired using extreme pairing. This means the top performer participates in two pairings.
* Special case: A single organism (r=1) pairs with itself (self-fertilization), producing two offspring.  

* **5.2.3.1. Locus Expression (Determining $x_j$ from a Locus Pair):** 
    * For each of the `n` dimensions, an expressed value $x_j$ is determined from the organism's corresponding locus pair (Locus A, Locus B).  
    * Let Locus A have the smaller `LocusAdjustment.Checksum` and Locus B have the larger (arbitrarily if equal).  
    * Normalize both checksums to the range [0.0, 1.0] by dividing by `m` (maximum possible value of a u64 from XXH3, i.e., $2^{64}-1$).  
    * Calculate $X_{midpoint}$ = midpoint of the two normalized checksums.  
    * Generate a pseudo-random number $R_{rand}$ in [0.0, 1.0].  
    * If $R_{rand} \le X_{midpoint}$, the `LocusValue` of Locus A (the one with the smaller original checksum) is expressed.  
    * Else, the `LocusValue` of Locus B is expressed.  
    * If original checksums are equal, there's a 50% chance of either locus's `LocusValue` being expressed.  

* **5.2.3.2. Gamete Creation and Crossover (Sexual Reproduction):** 
    * Each parent organism "shuffles" its two gametes (each gamete is a string of `n` loci).  
    * Shuffling uses "crossover": a predetermined constant number of random swap points are chosen, and segments are exchanged.  

* **5.2.3.3. Offspring DNA Formation:** 
    * Two offspring are formed from each pair. One of the crossed-over gametes is taken from each parent and recombined to produce a new organism; this is done twice to produce two offspring.
    * The offspring's loci are copies of the chosen parental loci. Mutations (Section 5.2.4) are applied to these copies.

**5.2.4. Mutation (Applied to Offspring's Loci during copying):** Parental loci are immutable. Mutations affect the offspring's loci. The mutation probabilities $m_1...m_5$ may themselves be evolvable dimensions.

* **5.2.4.1. Adjustment `DoublingOrHalvingFlag` Mutation:** 
    * Probability: $m_3$ (e.g., 0.1%).  
    * Action: Flips the boolean value.  
    * This mutation also occurs if `DirectionOfTravel` mutates.  

* **5.2.4.2. Adjustment `DirectionOfTravel` Mutation:** 
    * Probability: $m_4$ (e.g., 0.1%).  
    * Action: Flips from + to - or - to +.  
    * Triggers a `DoublingOrHalvingFlag` mutation.  

* **5.2.4.3. Adjustment `AdjustmentValue` Mutation:** 
    * Probability: $m_5$ (e.g., 0.1%).  
    * Action: If `DoublingOrHalvingFlag` is set to "doubling", `AdjustmentValue` doubles. If set to "halving", `AdjustmentValue` halves.  

* **5.2.4.4. Locus `ApplyAdjustmentFlag` Mutation:** 
    * If current flag value is `false`: Probability $m_1$ (e.g., 10%) of flipping to `true`.  
    * If current flag value is `true`: Probability $m_2$ (e.g., 50%) of flipping to `false`.  

* **5.2.4.5. Application of Adjustment to LocusValue:** 
    * If an offspring's locus has its `ApplyAdjustmentFlag` set to `true` (either inherited as true or mutated to true), its `LocusAdjustment` is applied to its `LocusValue` at the point the locus is copied.  
    * Application: `LocusValue = LocusValue + (DirectionOfTravel_sign * AdjustmentValue)`.

**5.2.5. Offspring Placement and Region Management:** 
    * New offspring are placed into the n-dimensional space based on their (expressed) coordinates and assigned to appropriate regions.  

* **Region Recalculation Triggers:** 
    * If any offspring falls outside the current master bounding box: Recalculate the master bounding box, then recalculate all regions and reassign all organisms.  
    * If previously empty regions become populated or populated regions become empty (due to births/deaths): Recalculate carrying capacities for all populated regions using the fitness-based allocation system.  
    * If the number of populated regions exceeds `Z`: Trigger a full recalculation of regions (and subsequently carrying capacities).

**5.2.6. Ageing and Death:** 
* Remove any organisms that have reached their maximum age ($A_{max}$).  
* Increment the age of all surviving organisms by one round.

**5.2.7. Loop:** Repeat from Section 5.2.1 for the next round with another known problem & solution.

### 5.3. Termination and Solution Extraction

**5.3.1. Running Duration:** 
* The system requires `K` known sets of (inputs, target outputs).  
* The algorithm runs for `K` rounds, using one unique set per round.  
* Recycling through the `K` sets is a possibility for experimentation but is initially ignored to avoid overfitting.

**5.3.2. Solution Extraction (After `K` rounds):** 
* Take a random sample of `S` previously used (inputs, target outputs) sets.  
* Apply these `S` problem sets to the current final population of organisms.  
* Calculate a weighted average fitness score for each organism across these `S` samples.  
* Produce a single, globally sorted list of all organisms based on this weighted average fitness.  
* The organism at the top of this list is considered the best solution.  
* The final parameters ($x_1, ..., x_n$) are read from the **expressed values** of this best organism's locus pairs (not directly from the stored `LocusValue`s, due to the recessiveness mechanism).

## 6. Parameters and Constants

**6.1. System-Wide Constants (Hardcoded Configuration):** 
* `P`: Total target population size.  
* `Z`: Maximum number of regions ($Z \ge n^2$).  
* Number of crossovers per gamete during sexual reproduction (predetermined constant).  
* `K`: Number of unique problem sets (and thus number of rounds for main run).  
* `S`: Size of the sample of problem sets used for final solution extraction.  
* Seed for the pseudo-random number generator (configurable via GlobalConstants).

**6.2. Evolvable Parameters (Part of the `n` dimensions):** 
* Mutation probabilities: $m_1, m_2, m_3, m_4, m_5$.  
* Maximum organism age: $A_{max}$ (must be a bounded parameter).  
* Problem-specific parameters (e.g., neural network weights and biases).

**6.2.1. System Parameter Evolution:**
* System parameters (mutation probabilities and maximum age) evolve genetically through the same inheritance, crossover, and mutation mechanisms as problem-specific parameters.
* However, system parameters do **not** influence spatial positioning or region assignment—only problem-specific parameters determine an organism's location in the niching space.
* System parameter evolution occurs **indirectly**: organisms with system parameter values that help them succeed in the problem space will have more reproductive opportunities, causing beneficial system parameter combinations to proliferate.
* This design ensures that evolutionary pressure on system parameters derives from their effectiveness at solving the actual problem, rather than from arbitrary spatial competition based on mutation rate values.
* System parameters typically evolve more slowly than problem-specific parameters, which is often desirable for meta-evolutionary stability.

**6.3. Derived Values:** * `n`: Total number of dimensions to be optimized.  
* $\hat{P}$: Number of distinct points in space occupied by organisms (distinctness by bit-wise equality).

## 7. Underlying Mechanisms and Assumptions

**7.1. Floating-Point Numbers:** 
* All coordinate values, locus values, and adjustment values are floating-point numbers.  
* Standard floating-point arithmetic is used.  
* Distinct points for $\hat{P}$ calculation are determined by precise (bit-wise) equality.  
* Region assignment uses precise inequalities.

**7.2. Pseudo-Random Number Generation:** 
* All random numbers (for initialization, mutation choices, crossover points, locus expression tie-breaking) are provided by a pseudo-random number generator (PRNG).  
* The PRNG is seeded with a configurable value (via GlobalConstants) to ensure repeatability of runs.

**7.3. Hashing:** 
* The XXH3 algorithm is used to generate 64-bit positive integer checksums for `LocusAdjustment` states.

## 8. Public API and Usage Patterns

The hill_descent_lib implementation provides a Rust API that abstracts the internal algorithm details described in this document. This section documents the key types and usage patterns.

### 8.1. TrainingData Enum

The `TrainingData` enum unifies the representation of different optimization scenarios:

```rust
pub enum TrainingData<'a> {
    /// Standard single-valued function optimization
    /// No external training data required
    None { floor_value: f64 },
    
    /// Supervised learning with training examples
    /// For functions requiring input/output pairs
    Supervised {
        inputs: &'a [Vec<f64>],
        outputs: &'a [Vec<f64>],
    },
}
```

**8.1.1. TrainingData::None Pattern:**
* Used for optimizing `SingleValuedFunction` implementations
* Represents scenarios where no external data is needed (e.g., mathematical function minimization)
* `floor_value`: The target minimum fitness value (typically 0.0)
* Corresponds to Section 2.2 when optimizing inherent function properties

**8.1.2. TrainingData::Supervised Pattern:**
* Used for optimizing `WorldFunction` implementations requiring external data
* Represents supervised learning scenarios (e.g., neural network training)
* `inputs`: Training input samples
* `outputs`: Expected output values for corresponding inputs
* Corresponds to Section 2.2 with known target outputs $(A_1, ..., A_p)$
* The fitness function internally compares function outputs to these targets

### 8.2. Core API Methods

**8.2.1. World::training_run()**

Executes one evolutionary epoch (corresponds to one iteration of Section 5.2):

```rust
pub fn training_run(&mut self, data: TrainingData) -> bool
```

* Evaluates all organisms using the provided training data
* Performs selection, reproduction, and region management
* Returns `true` if spatial resolution limit reached, `false` otherwise
* Typically called in a loop for multiple epochs

**8.2.2. World::get_best_organism()**

Extracts the best solution after training (corresponds to Section 5.3.2):

```rust
pub fn get_best_organism(&mut self, data: TrainingData) -> Arc<Organism>
```

* Re-evaluates population with provided training data
* Returns organism with lowest fitness score
* Extract parameters via `organism.phenotype().expression_problem_values()`

**8.2.3. World::get_best_params()**

Convenience method for parameter extraction:

```rust
pub fn get_best_params(&self) -> Vec<f64>
```

* Returns the problem parameters of the current best organism
* Non-mutating (uses cached scores)
* Excludes system parameters (mutation rates, max age)
* Returns expressed values from phenotype (per Section 4.1.2)

**8.2.4. World::get_best_score()**

Query current optimization progress:

```rust
pub fn get_best_score(&self) -> f64
```

* Returns minimum fitness across all organisms
* Returns `f64::MAX` if no organisms have been scored
* Non-mutating query of cached fitness values

### 8.3. Usage Pattern Examples

**8.3.1. Standard Optimization (No External Data):**

```rust
use hill_descent_lib::{setup_world, TrainingData, GlobalConstants};

let mut world = setup_world(&param_bounds, constants, Box::new(MyFunction));

for _epoch in 0..100 {
    world.training_run(TrainingData::None { floor_value: 0.0 });
}

let best_params = world.get_best_params();
```

**8.3.2. Supervised Learning (With Training Data):**

```rust
let train_inputs = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
let train_outputs = vec![vec![5.0], vec![6.0]];

for _epoch in 0..100 {
    world.training_run(TrainingData::Supervised {
        inputs: &train_inputs,
        outputs: &train_outputs,
    });
}
```

### 8.4. Implementation Mapping

The public API maps to internal algorithm components as follows:

| API Concept | Internal PDD Section | Notes |
|-------------|---------------------|-------|
| `TrainingData::None` | Section 2.2 (single-valued) | Fitness = distance from floor |
| `TrainingData::Supervised` | Section 2.2 (multi-valued) | Fitness = Euclidean distance |
| `training_run()` | Section 5.2 (one round) | One evolutionary epoch |
| `get_best_organism()` | Section 5.3.2 (solution extraction) | Best organism by fitness |
| `World` | Section 3 (System Overview) | Contains population, regions |
| `GlobalConstants` | Section 6.1 | System-wide configuration |
| `SingleValuedFunction` | Section 2.1 (specialized) | No external inputs needed |
| `WorldFunction` | Section 2.1 (general) | Accepts external inputs |

### 8.5. API Design Rationale

**8.5.1. TrainingData Enum vs Method Variants:**
* Unified interface prevents API duplication
* Type system enforces correct usage patterns
* Lifetime parameters (`'a`) prevent data copying
* Pattern matching makes intent explicit

**8.5.2. Mutation vs Non-Mutation:**
* `training_run()` and `get_best_organism()` are mutating (run epochs)
* `get_best_score()` and `get_best_params()` are non-mutating (query state)
* Clear separation between training and inspection operations

**8.5.3. Internal Data Management:**
* Training data is **not** stored in `World`
* Avoids large memory copies
* Users manage data lifetime
* Supports streaming/batching patterns
