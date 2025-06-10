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

The fitness function is defined as:  
$fitness(inputs, x_1, x_2, ..., x_n) = \sum_{i=1}^{p} (A_i - R_i)^2 + e_0$  

Where:  
* $(A_1, ..., A_p)$: The known target outputs for the given `inputs`.  
* $(R_1, ..., R_p)$: The outputs produced by the `NeuralNetwork` (or equivalent function) with the current parameters.  
* $e_0$: The smallest representable positive floating-point number. This addition prevents the fitness from being exactly zero, thus avoiding potential division-by-zero errors in subsequent calculations (e.g., carrying capacity).

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
    * This bounding box is recursively divided by halving it along each of these $n_p$ problem-specific dimensions.  
    * Let `x` be the number of currently populated n-orthotopes.  
    * Headroom for further division exists if $x \cdot 2^n < Z$.  
    * Division also stops if $x = \hat{P}$, where $\hat{P}$ is the number of distinct points in space occupied by organisms. Distinct points are defined by precise (bit-wise) equality of their n-dimensional coordinates. Further division is considered pointless at this stage.  
    * Unoccupied regions resulting from division are of no interest.

**4.2.3. Bounding Box:** 
* An n-dimensional hyperrectangle that encompasses all organisms in the population.  
* Initial Bounding Box: Calculated at startup to be twice the size (in each dimension) required to hold all initial organisms.  
* Recalculation: If any new organism (offspring) falls outside the current bounding box, a new bounding box is calculated (again, twice the required size in each dimension), and all regions are recalculated.

**4.2.4. Carrying Capacity ($P_i$):** 
* The number of organisms a region `i` can support.  
* Calculated for each populated region.  
* Dependent on the minimum fitness $F_i$ found within that region. $F_i$ is the fitness score of the fittest organism within region `i` (recall fitness includes $+ e_0$, so $F_i > 0$).  
* Formula: $P_i = P \cdot \frac{1/F_i}{\sum_{j=1}^{\hat{P}} (1/F_j)}$  
    * $P$: Total target population size.  
    * $F_i$: Minimum fitness in region `i`.  
    * The sum is over all $\hat{P}$ populated regions (where $\hat{P}$ is the number of distinct points, implying distinct regions if $x=\hat{P}$).  
* Recalculation: Required if previously empty regions become populated, previously populated regions become empty, or if the overall bounding box changes.

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
    3.  Order of encounter (first encountered is ranked higher).

**5.2.3. Reproduction (Occurs per Region):** 
* For each region, determine the required number of offspring, `r`, to fill the gap. This is based on the difference between the number of organisms that will be left after removing those at $A_{max}$ and the region's carrying capacity $P_i$. Organisms at $A_{max}$ don't count towards current population for capacity but can still reproduce.  

**Selection for Reproduction:** 
* Select the top `r` organisms from the region's ranked list.  
* If the number of organisms in the region (eligible for count) is less than `r`, all organisms in that region are selected.  

**Pairing and Reproduction Type:** 
* If `r` is even: All `r` selected organisms participate in sexual reproduction. They are paired sequentially from the ranked list (1st with 2nd, 3rd with 4th, etc.). Each pair produces two offspring.  
* If `r` is odd: The top-ranked organism reproduces asexually (producing one offspring). The remaining `r-1` organisms are paired sequentially for sexual reproduction.  

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
    * For sexual reproduction: Two offspring are formed. One of the crossed-over gametes is taken from each parent and recombined to produce a new organism; this is done twice to produce two offspring.  
    * For asexual reproduction: The single parent's two shuffled gametes are recombined with each other to form one offspring.  
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
    * If previously empty regions become populated or populated regions become empty (due to births/deaths): Recalculate carrying capacities for all populated regions.  
    * If the number of populated regions exceeds `Z`: Trigger a full recalculation of regions (and subsequently carrying capacities).

**5.2.6. Aging and Death:** 
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
* Seed for the pseudo-random number generator.

**6.2. Evolvable Parameters (Part of the `n` dimensions):** 
* Mutation probabilities: $m_1, m_2, m_3, m_4, m_5$.  
* Maximum organism age: $A_{max}$ (must be a bounded parameter).  
* Problem-specific parameters (e.g., neural network weights and biases).

**6.3. Derived Values:** * `n`: Total number of dimensions to be optimized.  
* $e_0$: Smallest representable positive floating-point number.  
* $\hat{P}$: Number of distinct points in space occupied by organisms (distinctness by bit-wise equality).

## 7. Underlying Mechanisms and Assumptions

**7.1. Floating-Point Numbers:** 
* All coordinate values, locus values, and adjustment values are floating-point numbers.  
* Standard floating-point arithmetic is used.  
* Distinct points for $\hat{P}$ calculation are determined by precise (bit-wise) equality.  
* Region assignment uses precise inequalities.

**7.2. Pseudo-Random Number Generation:** 
* All random numbers (for initialization, mutation choices, crossover points, locus expression tie-breaking) are provided by a pseudo-random number generator (PRNG).  
* The PRNG is seeded with a constant to ensure repeatability of runs.

**7.3. Hashing:** 
* The XXH3 algorithm is used to generate 64-bit positive integer checksums for `LocusAdjustment` states.